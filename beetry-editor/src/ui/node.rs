mod base;
mod context_menu;
mod parameter_dialog;
mod port;
mod renderer;

pub(super) mod control;
pub(super) mod leaf;
pub(super) mod root;

pub(crate) use base::Handlers;
pub(crate) use context_menu::{
    ContextMenu, Handlers as ContextMenuHandlers, State as ContextMenuState,
};
pub(crate) use parameter_dialog::{
    Dialog as ParameterDialog, Handlers as ParameterDialogHandlers, State as ParameterDialogState,
};
pub(crate) use port::input::Handlers as InputPortHandlers;
pub(crate) use port::output::Handlers as OutputPortHandlers;
pub(crate) use port::receiver::Handlers as ReceiverPortHandlers;
pub(crate) use port::sender::Handlers as SenderPortHandlers;

pub(crate) use renderer::Renderer;

use dioxus::prelude::*;

pub(crate) fn style_defs() -> Element {
    rsx! {
        {root::style_defs()}
        {control::style_defs()}
        {leaf::style_defs()}
        {port::style_defs()}
    }
}
