use std::time::Duration;

use anyhow::{Result, anyhow};
use beetry::runtime::{PeriodicTick, PeriodicTicker, TreeEngine, TreeEngineConfig};
#[expect(unused_imports, reason = "import all node and channel plugins")]
use beetry_example::domain::*;

#[tokio::test]
async fn ui_tree_runs_in_engine() -> Result<()> {
    let mut engine = TreeEngine::new(TreeEngineConfig::default())
        .tree_from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/domain/ui/tree.json"
        ))?
        .start_executor()?;
    let ticker = PeriodicTicker::new(PeriodicTick::new(Duration::from_millis(10)));

    tokio::time::timeout(Duration::from_secs(1), engine.tick_till_terminal(ticker))
        .await
        .map_err(|_| anyhow!("ui tree execution timed out"))??;

    Ok(())
}
