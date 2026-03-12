use anyhow::{Result, anyhow};
use beetry_core::{PeriodicTick, PeriodicTicker};
use beetry_engine::{TreeEngine, TreeEngineConfig};
use beetry_example::ParkingMilestone;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{Layer, Registry};

#[derive(Clone)]
struct EventCaptureLayer {
    events: Arc<Mutex<Vec<String>>>,
}

impl<S> Layer<S> for EventCaptureLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let target = event.metadata().target();
        if !target.starts_with("beetry_example::domain::parking::nodes") {
            return;
        }

        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        if let Some(message) = visitor.message {
            let mut guard = self
                .events
                .lock()
                .expect("event log lock should not be poisoned");
            guard.push(message);
        }
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}").trim_matches('"').to_string());
        }
    }
}

fn init_global_tracing() -> Arc<Mutex<Vec<String>>> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let subscriber = Registry::default().with(EventCaptureLayer {
        events: Arc::clone(&events),
    });
    tracing::subscriber::set_global_default(subscriber)
        .expect("global tracing subscriber should be initialized once");
    Arc::clone(&events)
}

fn extract_milestones(events: &[String]) -> Vec<ParkingMilestone> {
    let mut milestones = Vec::new();

    for event in events {
        let Some(milestone) = ParkingMilestone::from_log_message(event) else {
            continue;
        };
        milestones.push(milestone);
    }

    milestones
}

#[tokio::test(flavor = "multi_thread")]
async fn parking_tree_execution_matches_milestones() -> Result<()> {
    let events = init_global_tracing();
    let mut engine = TreeEngine::new(TreeEngineConfig::default())
        .tree_from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/domain/parking/tree.json"
        ))?
        .start_executor()?;
    let ticker = PeriodicTicker::new(PeriodicTick::new(Duration::from_millis(10)));
    tokio::time::timeout(Duration::from_secs(2), engine.tick_till_terminal(ticker))
        .await
        .map_err(|_| anyhow!("tree execution timed out"))??;

    let captured = events
        .lock()
        .expect("event log lock should not be poisoned")
        .clone();
    let milestones = extract_milestones(&captured);

    let expected = vec![
        ParkingMilestone::CheckReady(false),
        ParkingMilestone::CheckReady(true),
        ParkingMilestone::DetectStart,
        ParkingMilestone::DetectSuccess,
        ParkingMilestone::SelectStart,
        ParkingMilestone::SelectSuccess,
        ParkingMilestone::PlanStart,
        ParkingMilestone::PlanSuccess,
        ParkingMilestone::FollowStart,
        ParkingMilestone::FollowProgress(1),
        ParkingMilestone::FollowProgress(2),
        ParkingMilestone::FollowProgress(3),
        ParkingMilestone::FollowProgress(4),
        ParkingMilestone::FollowSuccess,
        ParkingMilestone::VerifyPoseTrue,
        ParkingMilestone::VerifyClearTrue,
        ParkingMilestone::ConfirmParkedTrue,
    ];

    assert_eq!(milestones, expected, "captured milestone sequence mismatch");
    Ok(())
}
