use std::time::Duration;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};
use tracing_tree::HierarchicalLayer;

use anyhow::Result;
use beetry_core::{PeriodicTick, PeriodicTicker};
use beetry_engine::{TreeEngine, TreeEngineConfig};
#[expect(unused_imports, reason = "import all node and channel plugins")]
use beetry_example::*;

#[tokio::main]
async fn main() -> Result<()> {
    let targets = Targets::new()
        .with_target("zbus", LevelFilter::ERROR)
        .with_default(LevelFilter::DEBUG);
    let subscriber = Registry::default().with(
        HierarchicalLayer::new(2)
            .with_targets(true)
            .with_filter(targets),
    );
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mut engine = TreeEngine::new(TreeEngineConfig::default())
        .tree_from_dialog()
        .await?
        .start_executor()?;
    let ticker = PeriodicTicker::new(PeriodicTick::new(Duration::from_millis(10)));
    engine.tick_till_terminal(ticker).await?;

    Ok(())
}
