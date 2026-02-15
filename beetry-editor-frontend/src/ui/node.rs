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
    Dialog as ParameterDialog, Handlers as ParameterDialogHandlers, Mode as ParameterDialogMode,
    State as ParameterDialogState,
};
pub use port::ConnectionOrigin;
pub use port::context_menu as port_context_menu;
pub(crate) use port::context_menu::Handlers as PortContextMenuHandlers;
pub use port::input::Handlers as InputPortHandlers;
pub use port::output::Handlers as OutputPortHandlers;
pub use port::receiver::Handlers as ReceiverPortHandlers;
pub use port::sender::Handlers as SenderPortHandlers;
pub use renderer::Renderer;

use crate::Point;
use dioxus::prelude::*;

pub const PARAM_DIALOG_POSITION: Point = Point { x: 300.0, y: 200.0 };

pub fn style_defs() -> Element {
    rsx! {
        {root::style_defs()}
        {control::style_defs()}
        {leaf::style_defs()}
        {port::style_defs()}
    }
}
