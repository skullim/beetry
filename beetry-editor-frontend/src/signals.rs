use std::marker::PhantomData;

use dioxus::prelude::{ReadableExt, Signal, WritableExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderTrigger<Tag> {
    // dummy flag to trigger value change
    flag: Signal<bool>,
    _tag: PhantomData<Tag>,
}

impl<Tag> RenderTrigger<Tag> {
    pub fn new() -> Self {
        Self {
            flag: Signal::new(false),
            _tag: PhantomData,
        }
    }

    pub fn request(&mut self) {
        self.flag.with_mut(|write| *write = !*write);
    }

    pub fn track(&self) {
        let _ = self.flag.read();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRenderTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeRenderTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelRenderTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelEdgeRenderTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortRenderTag;

pub type RequestNodeRender = RenderTrigger<NodeRenderTag>;
pub type RequestEdgeRender = RenderTrigger<EdgeRenderTag>;
pub type RequestChannelRender = RenderTrigger<ChannelRenderTag>;
pub type RequestChannelEdgeRender = RenderTrigger<ChannelEdgeRenderTag>;
pub type RequestPortRender = RenderTrigger<PortRenderTag>;
