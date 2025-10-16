use crate::{
    node::{Node, NodeIdentifier, NodeType, TracedNode},
    node_impl,
    status::TreeStatus,
};

pub trait Behavior {
    fn cond(&mut self) -> bool;
    fn reset(&mut self) {}
}

impl Behavior for Box<dyn Behavior> {
    fn cond(&mut self) -> bool {
        (**self).cond()
    }
    fn reset(&mut self) {
        (**self).reset();
    }
}

struct ConditionLeaf<B>
where
    B: Behavior,
{
    behavior: B,
}

impl<B> ConditionLeaf<B>
where
    B: Behavior,
{
    fn new(behavior: B) -> Self {
        Self { behavior }
    }
}

impl<B> Node for ConditionLeaf<B>
where
    B: Behavior,
{
    fn tick(&mut self) -> TreeStatus {
        match self.behavior.cond() {
            true => TreeStatus::Success,
            false => TreeStatus::Failure,
        }
    }
    fn reset(&mut self) {
        self.behavior.reset();
    }
}

pub(crate) struct Condition<B>(TracedNode<ConditionLeaf<B>>)
where
    B: Behavior;
impl<B> Condition<B>
where
    B: Behavior,
{
    pub(crate) fn new(behavior: B) -> Self {
        Self(TracedNode::new(
            ConditionLeaf::new(behavior),
            NodeIdentifier::new(NodeType::Condition),
        ))
    }
}
node_impl!(Condition<B> where B: Behavior);
