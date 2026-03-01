use anyhow::{Result, anyhow};
use beetry_core::{ActionBehavior, NodeTask, Sender, Task, TickStatus};
use beetry_editor_types::spec::node::{
    FieldDefinition, FieldMetadata, FieldName, FieldTypeSpec, ParamsSpec, ProvideParamSpec,
};
use beetry_plugin::action;
use beetry_reconstruction::ParamsReconstructor;
use mitsein::iter1::IntoIterator1;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{
    Receiver as TokioReceiver, Sender as TokioSender, channel as mpsc_channel,
};
use tracing::info;

use crate::Pose;

use super::super::messages::{BrakeState, ProximityState, VehicleState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishInterval {
    interval_ms: u64,
}

impl Default for PublishInterval {
    fn default() -> Self {
        Self { interval_ms: 100 }
    }
}

impl PublishInterval {
    pub(crate) fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
}

impl ProvideParamSpec for PublishInterval {
    fn provide() -> ParamsSpec {
        [(
            FieldName::from("interval_ms"),
            FieldDefinition {
                type_spec: FieldTypeSpec::U64(FieldMetadata::new(Arc::new(|value| {
                    if *value == 0 {
                        Err(anyhow!("interval must be greater than 0 ms"))
                    } else {
                        Ok(())
                    }
                }))),
                description: Some("Publisher period in milliseconds".into()),
            },
        )]
        .into_iter1()
        .collect1()
    }
}

struct PublishVehicleStateTask {
    send: TokioSender<VehicleState>,
    tick: u64,
    interval: Duration,
}

impl PublishVehicleStateTask {
    fn new(send: TokioSender<VehicleState>, interval: Duration) -> Self {
        Self {
            send,
            tick: 0,
            interval,
        }
    }
}

impl Task for PublishVehicleStateTask {
    async fn run(mut self) -> TickStatus {
        info!("VehicleStatePublisher task started");
        loop {
            self.tick += 1;
            let state = VehicleState {
                ready: self.tick >= 1,
                parked: self.tick >= 8,
                speed_mps: if self.tick >= 8 { 0.0 } else { 1.0 },
            };
            info!("VehicleStatePublisher publish: {:?}", state);
            if self.send.send(state).await.is_err() {
                info!("VehicleStatePublisher task stopping: receiver disconnected");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

pub struct VehicleStatePublisher<S> {
    send: S,
    recv: Option<TokioReceiver<VehicleState>>,
    interval: Duration,
}

impl<S> VehicleStatePublisher<S>
where
    S: Sender<VehicleState>,
{
    pub fn new(send: S) -> Self {
        Self::new_with_params(send, PublishInterval::default())
    }

    pub fn new_with_params(send: S, params: PublishInterval) -> Self {
        Self {
            send,
            recv: None,
            interval: params.interval(),
        }
    }
}

impl<S> ActionBehavior for VehicleStatePublisher<S>
where
    S: Sender<VehicleState>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let (send, recv) = mpsc_channel(8);
        self.recv = Some(recv);
        Ok(NodeTask::new(PublishVehicleStateTask::new(
            send,
            self.interval,
        )))
    }

    fn on_running(&mut self) {
        if let Some(recv) = &mut self.recv {
            while let Ok(state) = recv.try_recv() {
                let _ = self.send.try_send(state);
            }
        }
    }

    fn reset(&mut self) {
        self.recv = None;
    }

    fn on_aborted(&mut self) {
        self.recv = None;
    }
}

action! {
    VehicleStatePublisherPlugin: "VehicleStatePublisher";
    params(parameters): PublishInterval::provide();
    senders: [send: VehicleState => "Vehicle state"];
    create: VehicleStatePublisher::new_with_params(send, ParamsReconstructor::reconstruct(parameters)?);
}

pub struct LocalizationPublisher<S> {
    send: S,
    recv: Option<TokioReceiver<Pose>>,
    interval: Duration,
}

struct PublishLocalizationTask {
    send: TokioSender<Pose>,
    x: f32,
    interval: Duration,
}

impl PublishLocalizationTask {
    fn new(send: TokioSender<Pose>, interval: Duration) -> Self {
        Self {
            send,
            x: 0.0,
            interval,
        }
    }
}

impl Task for PublishLocalizationTask {
    async fn run(mut self) -> TickStatus {
        info!("LocalizationPublisher task started");
        loop {
            self.x += 1.0;
            let pose = Pose::new(self.x, 0.0);
            info!("LocalizationPublisher publish: {:?}", pose);
            if self.send.send(pose).await.is_err() {
                info!("LocalizationPublisher task stopping: receiver disconnected");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

impl<S> LocalizationPublisher<S>
where
    S: Sender<Pose>,
{
    pub fn new(send: S) -> Self {
        Self::new_with_params(send, PublishInterval::default())
    }

    pub fn new_with_params(send: S, params: PublishInterval) -> Self {
        Self {
            send,
            recv: None,
            interval: params.interval(),
        }
    }
}

impl<S> ActionBehavior for LocalizationPublisher<S>
where
    S: Sender<Pose>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let (send, recv) = mpsc_channel(8);
        self.recv = Some(recv);
        Ok(NodeTask::new(PublishLocalizationTask::new(
            send,
            self.interval,
        )))
    }

    fn on_running(&mut self) {
        if let Some(recv) = &mut self.recv {
            while let Ok(pose) = recv.try_recv() {
                let _ = self.send.try_send(pose);
            }
        }
    }

    fn reset(&mut self) {
        self.recv = None;
    }

    fn on_aborted(&mut self) {
        self.recv = None;
    }

    fn on_failure(&mut self) {
        self.recv = None;
    }
}

action! {
    LocalizationPublisherPlugin: "LocalizationPublisher";
    params(parameters): PublishInterval::provide();
    senders: [send: Pose => "Current pose"];
    create: LocalizationPublisher::new_with_params(send, ParamsReconstructor::reconstruct(parameters)?);
}

pub struct ProximityPublisher<S> {
    send: S,
    recv: Option<TokioReceiver<ProximityState>>,
    interval: Duration,
}

struct PublishProximityTask {
    send: TokioSender<ProximityState>,
    interval: Duration,
}

impl PublishProximityTask {
    fn new(send: TokioSender<ProximityState>, interval: Duration) -> Self {
        Self { send, interval }
    }
}

impl Task for PublishProximityTask {
    async fn run(self) -> TickStatus {
        info!("ProximityPublisher task started");
        loop {
            let state = ProximityState { blocked: false };
            info!("ProximityPublisher publish: {:?}", state);
            if self.send.send(state).await.is_err() {
                info!("ProximityPublisher task stopping: receiver disconnected");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

impl<S> ProximityPublisher<S>
where
    S: Sender<ProximityState>,
{
    pub fn new(send: S) -> Self {
        Self::new_with_params(send, PublishInterval::default())
    }

    pub fn new_with_params(send: S, params: PublishInterval) -> Self {
        Self {
            send,
            recv: None,
            interval: params.interval(),
        }
    }
}

impl<S> ActionBehavior for ProximityPublisher<S>
where
    S: Sender<ProximityState>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let (send, recv) = mpsc_channel(8);
        self.recv = Some(recv);
        Ok(NodeTask::new(PublishProximityTask::new(
            send,
            self.interval,
        )))
    }

    fn on_running(&mut self) {
        if let Some(recv) = &mut self.recv {
            while let Ok(alert) = recv.try_recv() {
                let _ = self.send.try_send(alert);
            }
        }
    }

    fn reset(&mut self) {
        self.recv = None;
    }

    fn on_aborted(&mut self) {
        self.recv = None;
    }

    fn on_failure(&mut self) {
        self.recv = None;
    }
}

action! {
    ProximityPublisherPlugin: "ProximityPublisher";
    params(parameters): PublishInterval::provide();
    senders: [send: ProximityState => "Proximity alert"];
    create: ProximityPublisher::new_with_params(send, ParamsReconstructor::reconstruct(parameters)?);
}

pub struct BrakePublisher<S> {
    send: S,
    recv: Option<TokioReceiver<BrakeState>>,
    interval: Duration,
}

struct PublishBrakeTask {
    send: TokioSender<BrakeState>,
    interval: Duration,
}

impl PublishBrakeTask {
    fn new(send: TokioSender<BrakeState>, interval: Duration) -> Self {
        Self { send, interval }
    }
}

impl Task for PublishBrakeTask {
    async fn run(self) -> TickStatus {
        info!("BrakePublisher task started");
        loop {
            let state = BrakeState { engaged: false };
            info!("BrakePublisher publish: {:?}", state);
            if self.send.send(state).await.is_err() {
                info!("BrakePublisher task stopping: receiver disconnected");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

impl<S> BrakePublisher<S>
where
    S: Sender<BrakeState>,
{
    pub fn new(send: S) -> Self {
        Self::new_with_params(send, PublishInterval::default())
    }

    pub fn new_with_params(send: S, params: PublishInterval) -> Self {
        Self {
            send,
            recv: None,
            interval: params.interval(),
        }
    }
}

impl<S> ActionBehavior for BrakePublisher<S>
where
    S: Sender<BrakeState>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let (send, recv) = mpsc_channel(8);
        self.recv = Some(recv);
        Ok(NodeTask::new(PublishBrakeTask::new(send, self.interval)))
    }

    fn on_running(&mut self) {
        if let Some(recv) = &mut self.recv {
            while let Ok(state) = recv.try_recv() {
                let _ = self.send.try_send(state);
            }
        }
    }

    fn reset(&mut self) {
        self.recv = None;
    }

    fn on_aborted(&mut self) {
        self.recv = None;
    }

    fn on_failure(&mut self) {
        self.recv = None;
    }
}

action! {
    BrakePublisherPlugin: "BrakePublisher";
    params(parameters): PublishInterval::provide();
    senders: [send: BrakeState => "Emergency brake state"];
    create: BrakePublisher::new_with_params(send, ParamsReconstructor::reconstruct(parameters)?);
}
