use crate::Backend;
use crate::Point;
use crate::ui::tooltip::TooltipCard;
use beetry_editor_backend::api;
use beetry_editor_backend::api::ParameterValueQueryView;
use beetry_editor_types::id::NodeId;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub(crate) struct TooltipProps {
    pub(crate) visible: Signal<bool>,
    pub(crate) node_id: NodeId,
    pub(crate) anchor: Point,
}

#[component]
pub(crate) fn Tooltip(props: TooltipProps) -> Element {
    if !*props.visible.read() {
        return rsx! {};
    }

    let node_id = props.node_id;
    let mut lines = vec![format!("ID : {node_id}")];

    let backend = use_context::<Backend>();
    if let Ok(params) =
        backend.with_peek(|s| api::node::parameters::query(s).parameters(node_id).cloned())
    {
        lines.extend(
            params
                .into_iter()
                .map(|(field, value)| format!("{field} : {value}")),
        );
    }

    rsx! {
        TooltipCard { anchor: props.anchor, lines }
    }
}
