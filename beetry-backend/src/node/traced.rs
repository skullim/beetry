use tracing::instrument;

use crate::{
    Node, TreeStatus,
    node::{Identifiable, NodeIdentifier},
};

pub(crate) struct TracedNode<N>
where
    N: Node,
{
    inner: N,
    id: NodeIdentifier,
}

impl<N> TracedNode<N>
where
    N: Node,
{
    pub(crate) fn new(inner: N, id: NodeIdentifier) -> Self {
        Self { inner, id }
    }
}

impl<N> Node for TracedNode<N>
where
    N: Node,
{
    #[instrument(skip_all, fields(id=%self.id))]
    fn tick(&mut self) -> TreeStatus {
        self.inner.tick()
    }

    #[instrument(skip_all, fields(id=%self.id))]
    fn abort(&mut self) {
        self.inner.abort()
    }

    #[instrument(skip_all, fields(id=%self.id))]
    fn reset(&mut self) {
        self.inner.reset()
    }
}
impl<N> Identifiable for TracedNode<N>
where
    N: Node,
{
    fn id(&self) -> &NodeIdentifier {
        &self.id
    }
}

#[macro_export]
macro_rules! node_impl {
    ($name:ident) => {
        impl Node for $name {
            fn tick(&mut self) -> TreeStatus {
                self.0.tick()
            }

            fn abort(&mut self) {
                self.0.abort()
            }

            fn reset(&mut self) {
                self.0.reset()
            }
        }
    };
        ($name:ident < $($gen:ident),* > $(where $($bounds:tt)*)? ) => {
        impl<$($gen),*> Node for $name<$($gen),*>
        $(where $($bounds)*)?
        {
            fn tick(&mut self) -> TreeStatus {
                self.0.tick()
            }

            fn abort(&mut self) {
                self.0.abort()
            }

            fn reset(&mut self) {
                self.0.reset()
            }
        }
    };
}
