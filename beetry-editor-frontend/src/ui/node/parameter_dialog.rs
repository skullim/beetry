use crate::Point;
use crate::editor::ServiceContext;
use beetry_editor_backend::node::ParameterValueParser;
use beetry_editor_types::{
    id::NodeId,
    output::node::Parameters,
    spec::node::{FieldName, FieldTypeSpec, ParamsSpec},
};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Handlers {
    pub(crate) on_confirm: EventHandler<(NodeId, Parameters)>,
    pub(crate) on_cancel: EventHandler<()>,
}

impl Handlers {
    pub(crate) fn new(
        on_confirm: impl FnMut((NodeId, Parameters)) -> Result<()> + 'static,
        on_cancel: impl FnMut(()) + 'static,
    ) -> Self {
        Self {
            on_confirm: EventHandler::new(on_confirm),
            on_cancel: EventHandler::new(on_cancel),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        id: NodeId,
    },
}

#[derive(Debug, Props, Clone, PartialEq, Eq)]
pub struct DialogProps {
    state: Signal<State>,
}

#[component]
pub fn Dialog(props: DialogProps) -> Element {
    debug!("rendering parameter dialog");
    let (id, position, params_spec, node_name) = match *props.state.read() {
        State::Idle => return rsx! {},
        State::Visible { position, id } => {
            let service = use_context::<ServiceContext>();
            let read = service.service.read();
            let node_api = read.node_api();
            let spec_api = node_api.spec();

            (
                id,
                position,
                Rc::new(spec_api.params(id).unwrap().clone()),
                Rc::new(spec_api.name(id).unwrap().clone()),
            )
        }
    };

    let parameters = use_signal(Parameters::default);
    let handlers = use_context::<Handlers>();
    let spec = Rc::clone(&params_spec);
    let on_confirm = move |_| {
        if !are_param_values_set(&spec, &parameters.read()) {
            return;
        }

        let values = parameters.read();
        handlers.on_confirm.call((id, values.cloned()));
    };

    let on_cancel = move |_| {
        handlers.on_cancel.call(());
    };

    let param_fields = params_spec.iter().map(|(name, _)| {
        rsx!(
            div { margin_bottom: "16px",
                ParameterField { id, name: name.clone(), parameters }
            }
        )
    });

    rsx! {
        div {
            position: "fixed",
            top: "0",
            left: "0",
            width: "100vw",
            height: "100vh",
            background: "rgba(0,0,0,0.5)",
            z_index: "1000",
            onclick: on_cancel,

            div {
                position: "absolute",
                left: "{position.x}px",
                top: "{position.y}px",
                background: "white",
                border: "1px solid #ccc",
                border_radius: "8px",
                box_shadow: "0 4px 16px rgba(0,0,0,0.3)",
                min_width: "320px",
                max_width: "500px",
                padding: "20px",
                z_index: "1001",
                onclick: move |evt| evt.stop_propagation(),

                h3 { margin: "0 0 16px 0", "Configure Parameters for {node_name.clone()}" }
                {param_fields}

                div {
                    display: "flex",
                    justify_content: "flex-end",
                    gap: "8px",
                    margin_top: "20px",

                    button {
                        padding: "8px 16px",
                        border: "1px solid #ddd",
                        border_radius: "4px",
                        background: "white",
                        cursor: "pointer",
                        onclick: on_cancel,
                        "Cancel"
                    }

                    button {
                        padding: "8px 16px",
                        border: "1px solid #007acc",
                        border_radius: "4px",
                        background: "#007acc",
                        color: "white",
                        cursor: "pointer",
                        onclick: on_confirm,
                        "Confirm"
                    }
                }
            }
        }
    }
}

fn are_param_values_set(spec: &ParamsSpec, values: &Parameters) -> bool {
    spec.iter()
        .map(|(name, _)| values.get(name))
        .all(|o_val| o_val.is_some())
}

#[derive(Props, Clone, PartialEq)]
struct ParameterFieldProps2 {
    id: NodeId,
    name: FieldName,
    parameters: Signal<Parameters>,
}

#[component]
fn ParameterField(props: ParameterFieldProps2) -> Element {
    let mut parameters = props.parameters;

    let service = use_context::<ServiceContext>();
    let read = service.service.read();
    let node_api = read.node_api();
    let spec_api = node_api.spec();

    let spec = spec_api.spec_by_node_id_pub(props.id).unwrap();
    let params_spec = spec.params().as_ref().unwrap();
    let field_def = params_spec.get(&props.name).unwrap();

    let mut error_msg = use_signal::<Option<String>>(|| None);

    rsx! {
        label { display: "block", margin_bottom: "4px", font_weight: "bold", {props.name.as_str()} }

        if let Some(desc) = &field_def.description {
            div { font_size: "12px", color: "#666", margin_bottom: "4px", {desc.as_str()} }
        }



        //@todo avoid clone later
        match field_def.type_spec.clone() {
            FieldTypeSpec::Bool(meta) => {
                rsx! {
                    input {
                        r#type: "checkbox",
                        checked: false,
                        onchange: move |evt| {
                            match ParameterValueParser::parse(
                                &FieldTypeSpec::Bool(meta.clone()),
                                evt.value(),
                            ) {
                                Ok(val) => {
                                    parameters.with_mut(|write| write.insert(props.name.clone(), val));
                                    error_msg.set(None);
                                }
                                Err(e) => {
                                    parameters.with_mut(|write| write.remove(&props.name));
                                    error_msg.set(Some(e.to_string()));
                                }
                            }
                        },
                    }
                }
            }
            FieldTypeSpec::I64(meta) => {
                rsx! {
                    input {
                        r#type: "number",
                        value: 0i64,
                        width: "100%",
                        padding: "4px 8px",
                        border_radius: "4px",
                        oninput: move |evt| {
                            match ParameterValueParser::parse(
                                &FieldTypeSpec::I64(meta.clone()),
                                evt.value(),
                            ) {
                                Ok(val) => {
                                    parameters.with_mut(|write| write.insert(props.name.clone(), val));
                                    error_msg.set(None);
                                }
                                Err(e) => {
                                    parameters.with_mut(|write| write.remove(&props.name));
                                    error_msg.set(Some(e.to_string()));
                                }
                            }
                        },
                    }
                }
            }
            FieldTypeSpec::U64(meta) => {
                rsx! {
                    input {
                        r#type: "number",
                        value: 0u64,
                        width: "100%",
                        padding: "4px 8px",
                        border_radius: "4px",
                        oninput: move |evt| {
                            match ParameterValueParser::parse(
                                &FieldTypeSpec::U64(meta.clone()),
                                evt.value(),
                            ) {
                                Ok(val) => {
                                    parameters.with_mut(|write| write.insert(props.name.clone(), val));
                                    error_msg.set(None);
                                }
                                Err(e) => {
                                    parameters.with_mut(|write| write.remove(&props.name));
                                    error_msg.set(Some(e.to_string()));
                                }
                            }
                        },
                    }
                }
            }
            FieldTypeSpec::F64(meta) => {
                rsx! {
                    input {
                        r#type: "number",
                        step: "1.00",
                        value: 0.0f64,
                        width: "100%",
                        padding: "4px 8px",
                        border_radius: "4px",
                        oninput: move |evt| {
                            match ParameterValueParser::parse(
                                &FieldTypeSpec::F64(meta.clone()),
                                evt.value(),
                            ) {
                                Ok(val) => {
                                    parameters.with_mut(|write| write.insert(props.name.clone(), val));
                                    error_msg.set(None);
                                }
                                Err(e) => {
                                    parameters.with_mut(|write| write.remove(&props.name));
                                    error_msg.set(Some(e.to_string()));
                                }
                            }
                        },
                    }
                }
            }
            FieldTypeSpec::String(meta) => {
                rsx! {
                    input {
                        r#type: "text",
                        value: "",
                        width: "100%",
                        padding: "4px 8px",
                        border_radius: "4px",
                        oninput: move |evt| {
                            match ParameterValueParser::parse(
                                &FieldTypeSpec::String(meta.clone()),
                                evt.value(),
                            ) {
                                Ok(val) => {
                                    parameters.with_mut(|write| write.insert(props.name.clone(), val));
                                    error_msg.set(None);
                                }
                                Err(e) => {
                                    parameters.with_mut(|write| write.remove(&props.name));
                                    error_msg.set(Some(e.to_string()));
                                }
                            }
                        },
                    }
                }
            }
        }
        ParameterErrorDialog { error_msg }
    }
}

#[component]
fn ParameterErrorDialog(error_msg: Signal<Option<String>>) -> Element {
    let read = error_msg.read();
    let dialog = read.iter().map(|msg| {
        rsx! {
            p { color: "red", font_size: "0.8rem", margin_top: "4px", "{msg:?}" }
        }
    });

    rsx! {
        {dialog}
    }
}
