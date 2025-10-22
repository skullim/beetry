use beetry_channel::{AnyBoxReceiver, external::ReceiverRegistry};
use beetry_editor::ProjectData;
use beetry_serde::ser::channel::MessageHashProvider;
use beetry_serialization::{Deserializer, JsonDeserializer};
use rfd::FileHandle;
use std::{io::Read, time::Duration};
use tracing_subscriber::{
    Layer, Registry,
    filter::{LevelFilter, Targets},
    layer::SubscriberExt,
};
use tracing_tree::HierarchicalLayer;

use anyhow::{Result, anyhow};
use beetry_builder::Builder;
use beetry_core::{
    BehaviorTreeTicker, BoxNode, RegisterTask, Root, Sender, TaskControl, Tree, TreeEngine,
};
use beetry_example::{
    ChargeCommand, CheckBattery, CheckBatteryParams, Drive, DriveInput, ExternalData, Localize,
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

    let creation_type = BtCreationType::Editor;
    let bt = match creation_type {
        BtCreationType::Editor => bt_from_editor(&builder, receiver_registry).await?,
        BtCreationType::Code => bt_from_code(&builder)?,
    };

    let ticker = BehaviorTreeTicker::new(Duration::from_secs(1));
    let mut engine = TreeEngine::new().set_ticker(ticker).set_tree(bt);

    for _ in 0..2 {
        engine.tick_till_terminal(&mut ready_exec).await;
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
    T: TaskControl + 'static,
{
    let (loc_send, loc_recv) = beetry_channel::tokio::mpsc::channel(16);
    let localize = Localize::new(loc_send);
    let drive = Drive::new(DriveInput::builder().pose(loc_recv).build());
    let check = CheckBattery::new(CheckBatteryParams::default());

    Ok(builder.tree(Root::new(builder.sequence([
        builder.condition(check),
        builder.sequence([builder.action(localize), builder.action(drive)]),
    ]))))
}

async fn bt_from_editor<R, T>(
    builder: &Builder<R, T>,
    receiver_registry: ReceiverRegistry,
) -> Result<Tree<BoxNode>>
where
    R: RegisterTask<T> + 'static,
    T: TaskControl + 'static,
{
    use beetry_reconstruction::TreeReconstructor;

    let handle = select_import_file().await?;
    let mut file = std::fs::File::open(handle.path())?;
    let mut content_buffer = String::new();
    file.read_to_string(&mut content_buffer)?;
    let data: ProjectData = JsonDeserializer::deserialize(&content_buffer)?;
    let mut reconstructor = TreeReconstructor::with_receiver_registry(receiver_registry);
    reconstructor.try_reconstruct(data.tree, builder)
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
