mod base;
mod context_menu;
mod parameter_dialog;
mod port;
mod renderer;

pub(super) mod control;
pub(super) mod leaf;
pub(super) mod root;

pub use base::Handlers;
pub use context_menu::{ContextMenu, Handlers as ContextMenuHandlers, State as ContextMenuState};
pub use parameter_dialog::{
    Dialog as ParameterDialog, Handlers as ParameterDialogHandlers, State as ParameterDialogState,
};
pub use port::input::Handlers as InputPortHandlers;
pub use port::output::Handlers as OutputPortHandlers;
pub use port::receiver::Handlers as ReceiverPortHandlers;
pub use port::receiver::Handlers2 as ReceiverPortHandlers2;
pub use port::sender::Handlers as SenderPortHandlers;
pub use port::sender::Handlers2 as SenderPortHandlers2;
pub use renderer::{Renderer, Renderer2};

use dioxus::prelude::*;

pub fn style_defs() -> Element {
    rsx! {
        {root::style_defs()}
        {control::style_defs()}
        {leaf::style_defs()}
        {port::style_defs()}
    }
}
