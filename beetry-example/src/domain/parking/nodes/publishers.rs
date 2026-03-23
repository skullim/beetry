use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow, bail};
use beetry::{
    channel::Sender,
    leaf::{ActionBehavior, NodeTask, Task},
    plugin::{action, parameter, parameter::ProvideParamSpec},
    runtime::TickStatus,
};
use mitsein::iter1::IntoIterator1;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{
    Receiver as TokioReceiver, Sender as TokioSender, channel as mpsc_channel,
};
use tracing::info;

use super::super::messages::{BrakeState, ProximityState, VehicleState};
use crate::domain::Pose;

pub struct VehicleStatePublisher<S> {
    send: S,
    recv: Option<TokioReceiver<VehicleState>>,
    interval: Duration,
}

impl<S> VehicleStatePublisher<S>
where
    S: Sender<VehicleState>,
{
    pub fn new(send: S, params: &PublishInterval) -> Self {
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

    fn on_running(&mut self) -> Result<()> {
        let Some(task_vehicle_state_receiver) = &mut self.recv else {
            bail!("vehicle state receiver should have been set");
        };

        while let Ok(state) = task_vehicle_state_receiver.try_recv() {
            self.send
                .try_send(state)
                .map_err(|error| anyhow!("failed to send vehicle state: {error}"))?;
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.recv = None;
    }

    fn on_aborted(&mut self) -> Result<()> {
        self.recv = None;
        Ok(())
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
            if let Err(e) = self.send.send(state).await {
                info!("VehicleStatePublisher task failed due to: {e}");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

action! {
    VehicleStatePublisherPlugin: "VehicleStatePublisher";
    params(parameters): PublishInterval::provide();
    senders: [send: VehicleState => "Vehicle state"];
    create: VehicleStatePublisher::new(send, &parameter::Deserializer::deserialize(parameters)?);
}

pub struct LocalizationPublisher<S> {
    send: S,
    recv: Option<TokioReceiver<Pose>>,
    interval: Duration,
}

impl<S> LocalizationPublisher<S>
where
    S: Sender<Pose>,
{
    pub fn new(send: S, params: &PublishInterval) -> Self {
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

    fn on_running(&mut self) -> Result<()> {
        let Some(task_pose_receiver) = &mut self.recv else {
            bail!("localization receiver should have been set");
        };
        while let Ok(pose) = task_pose_receiver.try_recv() {
            self.send
                .try_send(pose)
                .map_err(|error| anyhow!("failed to send localization pose: {error}"))?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.recv = None;
    }

    fn on_aborted(&mut self) -> Result<()> {
        self.recv = None;
        Ok(())
    }

    fn on_failure(&mut self) -> Result<()> {
        self.recv = None;
        Ok(())
    }
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
            if let Err(e) = self.send.send(pose).await {
                info!("LocalizationPublisher task failed due to: {e}");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

action! {
    LocalizationPublisherPlugin: "LocalizationPublisher";
    params(parameters): PublishInterval::provide();
    senders: [send: Pose => "Current pose"];
    create: LocalizationPublisher::new(send, &parameter::Deserializer::deserialize(parameters)?);
}

pub struct ProximityPublisher<S> {
    send: S,
    recv: Option<TokioReceiver<ProximityState>>,
    interval: Duration,
}

impl<S> ProximityPublisher<S>
where
    S: Sender<ProximityState>,
{
    pub fn new(send: S, params: &PublishInterval) -> Self {
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

    fn on_running(&mut self) -> Result<()> {
        let Some(task_proximity_receiver) = &mut self.recv else {
            bail!("proximity receiver should have been set");
        };
        while let Ok(alert) = task_proximity_receiver.try_recv() {
            self.send
                .try_send(alert)
                .map_err(|error| anyhow!("failed to send proximity state: {error}"))?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.recv = None;
    }

    fn on_aborted(&mut self) -> Result<()> {
        self.recv = None;
        Ok(())
    }

    fn on_failure(&mut self) -> Result<()> {
        self.recv = None;
        Ok(())
    }
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
            if let Err(e) = self.send.send(state).await {
                info!("ProximityPublisher task failed due to: {e}");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

action! {
    ProximityPublisherPlugin: "ProximityPublisher";
    params(parameters): PublishInterval::provide();
    senders: [send: ProximityState => "Proximity alert"];
    create: ProximityPublisher::new(send, &parameter::Deserializer::deserialize(parameters)?);
}

pub struct BrakePublisher<S> {
    send: S,
    recv: Option<TokioReceiver<BrakeState>>,
    interval: Duration,
}

impl<S> BrakePublisher<S>
where
    S: Sender<BrakeState>,
{
    pub fn new(send: S, params: &PublishInterval) -> Self {
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

    fn on_running(&mut self) -> Result<()> {
        let Some(task_brake_receiver) = &mut self.recv else {
            bail!("brake receiver should have been set");
        };
        while let Ok(state) = task_brake_receiver.try_recv() {
            self.send
                .try_send(state)
                .map_err(|error| anyhow!("failed to send brake state: {error}"))?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.recv = None;
    }

    fn on_aborted(&mut self) -> Result<()> {
        self.reset();
        Ok(())
    }

    fn on_failure(&mut self) -> Result<()> {
        self.reset();
        Ok(())
    }
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
            if let Err(e) = self.send.send(state).await {
                info!("BrakePublisher task failed due to: {e}");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

action! {
    BrakePublisherPlugin: "BrakePublisher";
    params(parameters): PublishInterval::provide();
    senders: [send: BrakeState => "Emergency brake state"];
    create: BrakePublisher::new(send, &parameter::Deserializer::deserialize(parameters)?);
}

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

impl parameter::ProvideParamSpec for PublishInterval {
    fn provide() -> parameter::Spec {
        [(
            "interval_ms".into(),
            parameter::FieldDefinition {
                type_spec: parameter::FieldTypeSpec::U64(parameter::FieldMetadata::new(Arc::new(
                    |value| {
                        if *value == 0 {
                            Err(anyhow!("interval must be greater than 0 ms"))
                        } else {
                            Ok(())
                        }
                    },
                ))),
                description: Some("Publisher period in milliseconds".into()),
            },
        )]
        .into_iter1()
        .collect1()
    }
}
