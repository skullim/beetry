//! Reconstructs and executes an authored Beetry tree on the backend.
//!
//! Run with:
//! `cargo run -p beetry-example --bin tree_runtime`
//!
//! The runtime opens a native file picker. Select a tree JSON authored/exported
//! in the editor to reconstruct it and execute it.
//!
//! Example:
//! `beetry-example/src/domain/parking/tree.json`
use std::time::Duration;

use anyhow::Result;
use beetry_core::{PeriodicTick, PeriodicTicker};
use beetry_engine::{TreeEngine, TreeEngineConfig};
#[expect(unused_imports, reason = "import all node and channel plugins")]
use beetry_example::domain::*;
use tracing_subscriber::{
    Layer, Registry,
    filter::{LevelFilter, Targets},
    layer::SubscriberExt,
};
use tracing_tree::HierarchicalLayer;

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
