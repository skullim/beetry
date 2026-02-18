use beetry_channel::AnyBoxReceiver;
use beetry_channel::external::ReceiverRegistry;
use beetry_editor_types::persistence::ValidTree;
use beetry_editor_types::spec::message::MessageHashProvider;
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
    BoxNode, PeriodicTick, PeriodicTicker, RegisterTask, Root, Sender, TaskHandle, Tree, TreeEngine,
};
use beetry_example::{
    ChargeCommand, CheckBattery, CheckBatteryParams, Drive, DriveReceivers, ExternalData, Localize,
};
use beetry_exec::{Executor, ExecutorConfig};

#[tokio::main(flavor = "current_thread")]
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

    let (mut sender, receiver) = beetry_channel::tokio::mpsc::channel(10);
    sender.try_send(ExternalData::new(ChargeCommand::Stop, false))?;
    sender.try_send(ExternalData::new(ChargeCommand::Start, true))?;
    let mut receiver_registry = ReceiverRegistry::new();
    receiver_registry.register(ExternalData::hash(), AnyBoxReceiver::new(receiver));

    let executor = Executor::new(ExecutorConfig::default());
    let (mut ready_exec, registry) = executor.into_ready_with_registry();
    let builder = Builder::new(registry);

    let creation_type = BtCreationType::Code;
    let bt = match creation_type {
        BtCreationType::Editor => bt_from_editor(&builder, receiver_registry).await?,
        BtCreationType::Code => bt_from_code(&builder)?,
    };

    let mut engine = TreeEngine::new(bt);

    for _ in 0..2 {
        let ticker = PeriodicTicker::new(PeriodicTick::new(Duration::from_secs(1)));
        engine.tick_till_terminal(ticker, &mut ready_exec).await?;
    }

    Ok(())
}

enum BtCreationType {
    Code,
    Editor,
}

fn bt_from_code<R, T>(builder: &Builder<R, T>) -> Result<Tree<BoxNode>>
where
    R: RegisterTask<T> + 'static,
    T: TaskHandle + 'static,
{
    let (loc_send, loc_recv) = beetry_channel::tokio::mpsc::channel(16);
    let localize = Localize::new(loc_send);
    let drive = Drive::new(DriveReceivers::builder().pose(loc_recv).build());
    let check = CheckBattery::new(CheckBatteryParams::default());

    Ok(builder.tree(Root::new(builder.sequence([
        builder.condition(check),
        builder.sequence([builder.action(localize), builder.action(drive)]),
    ]))))
}

async fn select_import_file() -> Result<FileHandle> {
    use rfd;
    rfd::AsyncFileDialog::new()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .set_title("Select file to import")
        .pick_file()
        .await
        .ok_or_else(|| anyhow!("No file selected"))
}

async fn bt_from_editor<R, T>(
    builder: &Builder<R, T>,
    receiver_registry: ReceiverRegistry,
) -> Result<Tree<BoxNode>>
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
    let mut reconstructor = TreeReconstructor::with_receiver_registry(receiver_registry)?;
    reconstructor.try_reconstruct(valid_tree, builder)
}
