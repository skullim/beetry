# Defining Tree

After introducing the execution model, the next step is to define a concrete
tree in code and execute it.

As this is a 🐝 tree let's model one of its behaviors.

```text
Root
└── Sequence
    ├── DetectFlower
    ├── FlyTo
    └── CollectPollen
```

In code, this can be written as:

```rust,no_run
use std::time::Duration;

use anyhow::Result;
use beetry::{
    leaf::{ActionBehavior, NodeTask, Task},
    node::{Root, Sequence},
    runtime::{PeriodicTick, PeriodicTicker, TickStatus, Tree, TreeEngine, TreeEngineConfig},
};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct Pose {
    x: f32,
    y: f32,
    z: f32,
}

struct DetectFlower {
    send: mpsc::Sender<Pose>,
}

impl ActionBehavior for DetectFlower {
    fn task(&mut self) -> Result<NodeTask> {
        Ok(NodeTask::new(DetectFlowerTask {
            send: self.send.clone(),
        }))
    }
}

struct FlyTo {
    recv: mpsc::Receiver<Pose>,
}

impl ActionBehavior for FlyTo {
    fn task(&mut self) -> Result<NodeTask> {
        Ok(NodeTask::new(FlyToTask {
            pose: self.recv.try_recv().ok(),
        }))
    }
}

struct CollectPollen;

impl ActionBehavior for CollectPollen {
    fn task(&mut self) -> Result<NodeTask> {
        Ok(NodeTask::new(CollectPollenTask))
    }
}

struct DetectFlowerTask {
    send: mpsc::Sender<Pose>,
}

impl Task for DetectFlowerTask {
    async fn run(self) -> TickStatus {
        // Logic to detect a flower would have been be implemented here 
        if self.send.try_send(Pose {
            x: 1.3,
            y: 2.1,
            z: 0.7,
        }).is_err() {
            return TickStatus::Failure;
        }
        TickStatus::Success
    }
}

struct FlyToTask {
    pose: Option<Pose>,
}

impl Task for FlyToTask {
    async fn run(self) -> TickStatus {
        let _ = self.pose;
        // Logic to fly to pose would have been implemented here 
        TickStatus::Success
    }
}

struct CollectPollenTask;

impl Task for CollectPollenTask {
    async fn run(self) -> TickStatus {
        // Logic to collect pollen would have been implemented here 
        TickStatus::Success
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let (send, recv) = mpsc::channel(1);
    let engine = TreeEngine::new(TreeEngineConfig::default());

    let tree = Tree::new(Root::new(Sequence::new([
        Box::new(engine.register_action(DetectFlower { send })),
        Box::new(engine.register_action(FlyTo { recv })),
        Box::new(engine.register_action(CollectPollen)),
    ])));

    let mut engine = engine.tree(tree).start_executor()?;
    let ticker = PeriodicTicker::new(PeriodicTick::new(Duration::from_millis(10)));
    let status = engine.tick_till_terminal(ticker).await?;
    Ok(())
}
```

The example already covers a lot. It uses `ActionBehavior` to define
custom action logic, `Tree` to define the hierarchy of nodes, and
`TreeEngine` to register action nodes and execute the tree.

There are a few things worth keeping in mind.

First, Beetry does not force a specific inter-node communication.
In this example, a Tokio channel is used to send and receiver `Pose`, but the same tree could use a blackboard, shared state, or any other IPC-style mechanism if desired.

Second, the shape of the tree is small, so its
structure is easy to understand. Adding or removing one or two
nodes is not a major problem at this scale. In practical applications, however,
trees usually become much wider and deeper. Once that happens, reasoning about their structure in code becomes harder. In practice, trees of realistic size are usually designed in the editor
 (see [Editor](../editor.md) chapter for more details).

Using an editor makes it easier to reason about a tree, at the cost of boxing nodes and slightly restricting the constructor API, since arbitrary data cannot be passed directly.
