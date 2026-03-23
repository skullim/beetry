use anyhow::{Result, anyhow, bail};
use beetry::{
    channel::{Receiver, Sender},
    leaf::{ActionBehavior, ConditionBehavior, NodeTask, Task},
    plugin::{action, condition, parameter, parameter::ProvideParamSpec},
    runtime::TickStatus,
};
use tokio::sync::mpsc::{
    Receiver as TokioReceiver, Sender as TokioSender, channel as mpsc_channel,
};
use tracing::info;

use super::{
    super::messages::{BrakeState, ProximityState, SafetyStatus, VehicleState},
    ParkingMilestone,
    publishers::PublishInterval,
};

enum SafetyUpdate {
    Proximity(ProximityState),
    Brake(BrakeState),
}

pub struct SafetyMonitor<PR, BR, S> {
    proximity_recv: PR,
    brake_recv: BR,
    updates_send: Option<TokioSender<SafetyUpdate>>,
    status_recv: Option<TokioReceiver<SafetyStatus>>,
    send: S,
    interval: std::time::Duration,
}

impl<PR, BR, S> SafetyMonitor<PR, BR, S>
where
    PR: Receiver<ProximityState>,
    BR: Receiver<BrakeState>,
    S: Sender<SafetyStatus>,
{
    pub fn new(proximity_recv: PR, brake_recv: BR, send: S, params: &PublishInterval) -> Self {
        Self {
            proximity_recv,
            brake_recv,
            updates_send: None,
            status_recv: None,
            send,
            interval: params.interval(),
        }
    }
}

impl<PR, BR, S> ActionBehavior for SafetyMonitor<PR, BR, S>
where
    PR: Receiver<ProximityState>,
    BR: Receiver<BrakeState>,
    S: Sender<SafetyStatus>,
{
    fn task(&mut self) -> Result<NodeTask> {
        let (updates_send, updates_recv) = mpsc_channel(8);
        let (status_send, status_recv) = mpsc_channel(8);
        self.updates_send = Some(updates_send);
        self.status_recv = Some(status_recv);
        Ok(NodeTask::new(PublishSafetyStatusTask::new(
            updates_recv,
            status_send,
            self.interval,
        )))
    }

    fn on_running(&mut self) -> Result<()> {
        let Some(safety_update_sender) = &self.updates_send else {
            bail!("safety sender should have been set");
        };
        let Some(safety_status_receiver) = &mut self.status_recv else {
            bail!("safety status receiver should have been set");
        };

        while let Ok(v) = self.proximity_recv.try_recv() {
            safety_update_sender
                .try_send(SafetyUpdate::Proximity(v))
                .map_err(|error| anyhow!("failed to send proximity update: {error}"))?;
        }
        while let Ok(v) = self.brake_recv.try_recv() {
            safety_update_sender
                .try_send(SafetyUpdate::Brake(v))
                .map_err(|error| anyhow!("failed to send brake update: {error}"))?;
        }
        while let Ok(status) = safety_status_receiver.try_recv() {
            self.send
                .try_send(status)
                .map_err(|error| anyhow!("failed to send safety status: {error}"))?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.updates_send = None;
        self.status_recv = None;
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

struct PublishSafetyStatusTask {
    updates_recv: TokioReceiver<SafetyUpdate>,
    send: TokioSender<SafetyStatus>,
    last_proximity: ProximityState,
    last_brake: BrakeState,
    interval: std::time::Duration,
}

impl PublishSafetyStatusTask {
    fn new(
        updates_recv: TokioReceiver<SafetyUpdate>,
        send: TokioSender<SafetyStatus>,
        interval: std::time::Duration,
    ) -> Self {
        Self {
            updates_recv,
            send,
            last_proximity: ProximityState::default(),
            last_brake: BrakeState::default(),
            interval,
        }
    }
}

impl Task for PublishSafetyStatusTask {
    async fn run(mut self) -> TickStatus {
        info!("SafetyMonitor task started");
        loop {
            while let Ok(update) = self.updates_recv.try_recv() {
                match update {
                    SafetyUpdate::Proximity(v) => self.last_proximity = v,
                    SafetyUpdate::Brake(v) => self.last_brake = v,
                }
            }

            let safe = !self.last_proximity.blocked && !self.last_brake.engaged;
            info!(
                "SafetyMonitor publish: safe={}, proximity={:?}, brake={:?}",
                safe, self.last_proximity, self.last_brake
            );
            if let Err(e) = self.send.send(SafetyStatus { safe }).await {
                info!("SafetyMonitor task failed due to: {e}");
                return TickStatus::Failure;
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

action! {
    SafetyMonitorPlugin: "SafetyMonitor";
    params(parameters): PublishInterval::provide();
    receivers: [
        proximity_recv: ProximityState => "Proximity alert",
        brake_recv: BrakeState => "Emergency brake",
    ];
    senders: [send: SafetyStatus => "Safety status"];
    create: SafetyMonitor::new(
        proximity_recv,
        brake_recv,
        send,
        &parameter::Deserializer::deserialize(parameters)?,
    );
}

pub struct CheckSystemReady<R> {
    recv: R,
    last_state: VehicleState,
    last_emitted_ready: Option<bool>,
}

impl<R> CheckSystemReady<R>
where
    R: Receiver<VehicleState>,
{
    pub fn new(recv: R) -> Self {
        Self {
            recv,
            last_state: VehicleState::default(),
            last_emitted_ready: None,
        }
    }
}

impl<R> ConditionBehavior for CheckSystemReady<R>
where
    R: Receiver<VehicleState>,
{
    fn cond(&mut self) -> bool {
        if let Ok(v) = self.recv.try_recv() {
            self.last_state = v;
        }
        let ready = self.last_state.ready;
        info!(
            "CheckSystemReady evaluated: ready={}, state={:?}",
            ready, self.last_state
        );
        if self.last_emitted_ready != Some(ready) {
            ParkingMilestone::CheckReady(ready).emit();
            self.last_emitted_ready = Some(ready);
        }
        ready
    }
}

condition! {
    CheckSystemReadyPlugin: "CheckSystemReady";
    receivers: [recv: VehicleState => "Vehicle state"];
    create: CheckSystemReady::new(recv);
}
