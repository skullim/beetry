use crate::Backend;
use crate::ui::error::ErrorQueueState;
use crate::{Point, ui::handler::define_handlers};
use anyhow::Context;
use beetry_editor_backend::api::{ParameterValueParser, SpecByNodeIdQueryView};
use beetry_editor_types::output::node::ParameterValue;
use beetry_editor_types::{
    id::NodeId,
    output::node::Parameters,
    spec::node::{FieldName, FieldTypeSpec, ParamsSpec},
};
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;
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
    let State::Visible { position, id, mode } = *state.read() else {
        return rsx! {};
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
            let Some(parameters) = backend
                .with_peek(|s| -> anyhow::Result<Parameters> {
                    Ok(beetry_editor_backend::api::node::parameters::get(s, id)?.clone())
                })
                .map_err(|err| {
                    errors.push(format!(
                        "failed to load existing node parameters for {id}: {err}"
                    ));
                })
                .ok()
            else {
                return rsx! {};
            };

            parameters
        }
    };
    let parameters = use_signal(move || initial_parameters);

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
        rsx!(ParameterField {
            id,
            name: name.clone(),
            parameters
        })
    });

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
    let mut errors = use_context::<ErrorQueueState>();
    let id = props.id;

    let Some(field_def) = use_hook(|| {
        let read = backend.read();
        let spec_query = beetry_editor_backend::api::node::spec::by_node_id(&(*read));
        let field_def = (|| -> anyhow::Result<_> {
            let spec = spec_query.spec(id)?;
            let params_spec = spec.params().as_ref().context("node has no params spec")?;
            let field_def = params_spec
                .get(&props.name)
                .context("param field not found in params spec")?;
            Ok(Rc::new(field_def.clone()))
        })();
        match field_def {
            Ok(field_def) => Some(field_def),
            Err(err) => {
                errors.push(err);
                None
            }
        }
    }) else {
        return rsx! {};
    };

    let mut error_msg = use_signal::<Option<String>>(|| None);
    let field_name = props.name.clone();
    let field_def_for_handler = Rc::clone(&field_def);
    let make_parse_handler = move || {
        let field_def = field_def_for_handler;
        move |evt: Event<FormData>| match ParameterValueParser::parse(
            &field_def.type_spec,
            evt.value(),
        ) {
            Ok(val) => {
                parameters.with_mut(|write| write.insert(field_name.clone(), val));
                error_msg.set(None);
            }
            Err(e) => {
                error_msg.set(Some(e.to_string()));
            }
        }
    };

    let o_val = parameters.read().get(&props.name).cloned();
    rsx! {
        div { class: "bt-form-field",
            label { class: "bt-form-label", {props.name.as_str()} }

            if let Some(desc) = &field_def.description {
                div { class: "bt-form-description", {desc.as_str()} }
            }

            match &field_def.type_spec {
                FieldTypeSpec::Bool(_) => {
                    rsx! {
                        input {
                            class: "bt-form-checkbox",
                            r#type: "checkbox",
                            checked: o_val.map(ParameterValue::into_bool).unwrap_or_default(),
                            onchange: make_parse_handler(),
                        }
                    }
                }
                FieldTypeSpec::I64(_) => {
                    rsx! {
                        input {
                            class: "bt-form-input",
                            r#type: "number",
                            value: o_val.map(ParameterValue::into_i64).unwrap_or_default(),
                            oninput: make_parse_handler(),
                        }
                    }
                }
                FieldTypeSpec::U64(_) => {
                    rsx! {
                        input {
                            class: "bt-form-input",
                            r#type: "number",
                            value: o_val.map(ParameterValue::into_u64).unwrap_or_default(),
                            oninput: make_parse_handler(),
                        }
                    }
                }
                FieldTypeSpec::F64(_) => {
                    rsx! {
                        input {
                            class: "bt-form-input",
                            r#type: "number",
                            step: "1.00",
                            value: o_val.map(ParameterValue::into_f64).unwrap_or_default(),
                            oninput: make_parse_handler(),
                        }
                    }
                }
                FieldTypeSpec::String(_) => {
                    rsx! {
                        input {
                            class: "bt-form-input",
                            r#type: "text",
                            value: o_val.map(ParameterValue::into_string).unwrap_or_default(),
                            oninput: make_parse_handler(),
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
