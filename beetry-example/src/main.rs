use beetry_editor_types::persistence::ValidTree;
use beetry_serialization::{Deserializer, JsonDeserializer};
use rfd::FileHandle;
use std::io::Read;
use std::time::Duration;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};
use tracing_tree::HierarchicalLayer;

use anyhow::{Result, anyhow};
use beetry_builder::Builder;
use beetry_core::{
    BoxNode, ExecutorConcept, PeriodicTick, PeriodicTicker, RegisterTask, TaskHandle, Tree,
    TreeEngine,
};
use beetry_exec::{Executor, ExecutorConfig};

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

    let executor = Executor::new(ExecutorConfig::default());
    let (mut ready_exec, registry) = executor.into_ready_with_registry();
    let builder = Builder::new(registry);

    let _exec_thread = std::thread::spawn(move || -> anyhow::Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move { ready_exec.run().await })
    });
    let bt = bt_from_editor(&builder).await?;
    let mut engine = TreeEngine::new(bt);
    let ticker = PeriodicTicker::new(PeriodicTick::new(Duration::from_millis(10)));
    engine.tick_till_terminal(ticker).await?;

    Ok(())
}

async fn select_import_file() -> Result<FileHandle> {
    rfd::AsyncFileDialog::new()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .set_title("Select file to import")
        .pick_file()
        .await
        .ok_or_else(|| anyhow!("No file selected"))
}

async fn bt_from_editor<R, T>(builder: &Builder<R, T>) -> Result<Tree<BoxNode>>
where
    R: RegisterTask<T> + 'static,
    T: TaskHandle + 'static,
{
    use beetry_reconstruction::TreeReconstructor;

    let handle = select_import_file().await?;
    let mut file = std::fs::File::open(handle.path())?;
    let mut content_buffer = String::new();
    file.read_to_string(&mut content_buffer)?;
    let valid_tree: ValidTree = JsonDeserializer::deserialize(&content_buffer)?;
    let mut reconstructor = TreeReconstructor::new()?;
    reconstructor.try_reconstruct(valid_tree, builder)
}
