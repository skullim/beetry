use crate::Backend;
use crate::ui::error::ErrorQueueState;
use crate::{Point, ui::handler::define_handlers};
use beetry_editor_backend::api::{ParameterValueParser, SpecByNodeIdQueryView};
use beetry_editor_types::{
    id::NodeId,
    output::node::Parameters,
    spec::node::{FieldName, FieldTypeSpec, ParamsSpec},
};
use dioxus::prelude::*;
use dioxus_logger::tracing::{debug, error};
use std::rc::Rc;

pub const DEFAULT_DIALOG_POSITION: Point = Point { x: 300.0, y: 200.0 };

define_handlers!(on_confirm: (NodeId, Parameters),
          on_cancel: (),
);

#[derive(Debug, Default, Clone, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Visible {
        position: Point,
        id: NodeId,
        mode: Mode,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Create,
    Update,
}

#[component]
pub fn Dialog(state: ReadSignal<State>) -> Element {
    debug!("rendering");
    let (id, position, mode) = match *state.read() {
        State::Idle => return rsx! {},
        State::Visible { position, id, mode } => (id, position, mode),
    };

    rsx! {
        VisibleDialog { id, position, mode }
    }
}

#[derive(Debug, Props, Clone, PartialEq)]
struct VisibleDialogProps {
    id: NodeId,
    position: Point,
    mode: Mode,
}

#[component]
fn VisibleDialog(props: VisibleDialogProps) -> Element {
    let id = props.id;
    let position = props.position;
    let mode = props.mode;
    let backend = use_context::<Backend>();
    let read = backend.read();
    let spec_query = beetry_editor_backend::api::node::spec::by_node_id(&(*read));

    let mut errors = use_context::<ErrorQueueState>();
    let Some((params_spec, node_name)) = (|| -> anyhow::Result<_> {
        let params_spec = Rc::new(spec_query.params(id)?.clone());
        let node_name = Rc::new(spec_query.name(id)?.clone());
        Ok((params_spec, node_name))
    })()
    .map_err(|e| errors.push(e))
    .ok() else {
        return rsx! {};
    };

    let initial_parameters = match mode {
        Mode::Create => Parameters::default(),
        Mode::Update => {
            match backend.with_peek(|s| -> anyhow::Result<Parameters> {
                Ok(beetry_editor_backend::api::node::parameters::get(s, id)?.clone())
            }) {
                Ok(parameters) => parameters,
                Err(err) => {
                    error!("failed to load existing node parameters for {id}: {err}");
                    Parameters::default()
                }
            }
        }
    };
    let parameters = use_signal(move || initial_parameters.clone());
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

    let param_fields = params_spec
        .iter()
        .map(|(name, _)| rsx!(
            ParameterField { id, name: name.clone(), parameters }
        ));

    rsx! {
        div { class: "bt-dialog-overlay", onclick: on_cancel,

            div {
                class: "bt-dialog bt-dialog--parameter",
                left: "{position.x}px",
                top: "{position.y}px",
                onclick: move |evt| evt.stop_propagation(),

                h3 { class: "bt-dialog-title",
                    "Configure Parameters:"
                    br {}
                    span { class: "bt-dialog-subtitle", "{node_name}" }
                }
                {param_fields}

                div { class: "bt-dialog-actions",

                    button {
                        class: "bt-btn bt-btn--dialog-secondary",
                        onclick: on_cancel,
                        "Cancel"
                    }

                    button {
                        class: "bt-btn bt-btn--dialog-primary",
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
struct ParameterFieldProps {
    id: NodeId,
    name: FieldName,
    parameters: Signal<Parameters>,
}

#[component]
fn ParameterField(props: ParameterFieldProps) -> Element {
    let mut parameters = props.parameters;

    let backend = use_context::<Backend>();
    let read = backend.read();
    let spec_query = beetry_editor_backend::api::node::spec::by_node_id(&(*read));

    let mut errors = use_context::<ErrorQueueState>();
    let Some(field_def) = (|| -> anyhow::Result<_> {
        let spec = spec_query.spec(props.id)?;
        let params_spec = spec.params().as_ref().context("node has no params spec")?;
        let field_def = params_spec
            .get(&props.name)
            .context("param field not found in params spec")?;
        Ok(field_def)
    })()
    .map_err(|e| errors.push(e))
    .ok() else {
        return rsx! {};
    };

    let mut error_msg = use_signal::<Option<String>>(|| None);
    let o_val = parameters.read().get(&props.name).cloned();
    rsx! {
        div { class: "bt-form-field",
            label { class: "bt-form-label", {props.name.as_str()} }

            if let Some(desc) = &field_def.description {
                div { class: "bt-form-description", {desc.as_str()} }
            }

            //@todo avoid clone
            match field_def.type_spec.clone() {
                FieldTypeSpec::Bool(meta) => {
                    rsx! {
                        input {
                            class: "bt-form-checkbox",
                            r#type: "checkbox",
                            checked: o_val.map(|v| v.into_bool()).unwrap_or_default(),
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
                            class: "bt-form-input",
                            r#type: "number",
                            value: o_val.map(|v| v.into_i64()).unwrap_or_default(),
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
                            class: "bt-form-input",
                            r#type: "number",
                            value: o_val.map(|v| v.into_u64()).unwrap_or_default(),
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
                            class: "bt-form-input",
                            r#type: "number",
                            step: "1.00",
                            value: o_val.map(|v| v.into_f64()).unwrap_or_default(),
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
                            class: "bt-form-input",
                            r#type: "text",
                            value: o_val.map(|v| v.into_string()).unwrap_or_default(),
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
}

#[component]
fn ParameterErrorDialog(error_msg: Signal<Option<String>>) -> Element {
    let read = error_msg.read();
    let dialog = read.iter().map(|msg| {
        rsx! {
            p { class: "bt-form-error", "{msg:?}" }
        }
    });

    rsx! {
        {dialog}
    }
}
