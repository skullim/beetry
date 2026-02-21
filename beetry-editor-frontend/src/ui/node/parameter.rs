use crate::Backend;
use crate::ui::error::ErrorQueueState;
use crate::{Point, ui::handler::define_handlers};
use beetry_editor_backend::api::{ParameterValueParser, SpecByNodeIdQueryView};
use beetry_editor_types::output::node::ParameterValue;
use beetry_editor_types::spec::node::FieldDefinition;
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

    let mut errors = use_context::<ErrorQueueState>();
    let Some(param_spec_with_name) = use_hook(|| {
        (|| -> anyhow::Result<_> {
            let read = backend.read();
            let spec_query = beetry_editor_backend::api::node::spec::by_node_id(&*read);

            let params_spec = spec_query.params(id)?.clone();
            let node_name = spec_query.name(id)?.clone();

            Ok(CopyValue::new((params_spec, node_name)))
        })()
        .map_err(|e| errors.push(e))
        .ok()
    }) else {
        return rsx! {};
    };

    let parameters = match mode {
        Mode::Create => CopyValue::new(Parameters::default()),
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

            CopyValue::new(parameters)
        }
    };

    let handlers = use_context::<Handlers>();
    let on_confirm = move |_| {
        if !are_param_values_set(&param_spec_with_name.peek().0, &parameters.peek()) {
            return;
        }

        let values = parameters.peek();
        handlers.on_confirm.call((id, values.cloned()));
    };

    let on_cancel = move |_| {
        handlers.on_cancel.call(());
    };

    let field_data_container = use_hook(|| {
        let data: Vec<_> = param_spec_with_name
            .peek()
            .0
            .iter()
            .map(|(name, def)| {
                CopyValue::new(FieldData {
                    name: name.clone(),
                    def: def.clone(),
                })
            })
            .collect();
        Rc::new(data)
    });

    let param_fields = field_data_container
        .iter()
        .copied()
        .map(|data| rsx!(ParameterField { data, parameters }));

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
                    span { class: "bt-dialog-subtitle", "{param_spec_with_name.peek().1}" }
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

struct FieldData {
    name: FieldName,
    def: FieldDefinition,
}

#[derive(Props, Clone, PartialEq)]
struct ParameterFieldProps {
    data: CopyValue<FieldData>,
    parameters: CopyValue<Parameters>,
}

#[component]
fn ParameterField(props: ParameterFieldProps) -> Element {
    let mut parameters = props.parameters;

    let mut error_msg = use_signal::<Option<String>>(|| None);
    let make_parse_handler = move || {
        let data = props.data;
        move |evt: Event<FormData>| match ParameterValueParser::parse(
            &data.peek().def.type_spec,
            evt.value(),
        ) {
            Ok(val) => {
                parameters.with_mut(|p| p.insert(data.peek().name.clone(), val));
                error_msg.set(None);
            }
            Err(e) => {
                parameters.with_mut(|p| p.remove(&data.peek().name));
                error_msg.set(Some(e.to_string()));
            }
        }
    };
    let name = &props.data.peek().name;
    let def = &props.data.peek().def;
    let o_val = parameters.peek().get(name).cloned();
    rsx! {
        div { class: "bt-form-field",
            label { class: "bt-form-label", {name.as_str()} }

            if let Some(desc) = &def.description {
                div { class: "bt-form-description", {desc.as_str()} }
            }

            match &def.type_spec {
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
                            r#type: "text",
                            inputmode: "numeric",
                            value: o_val.map(ParameterValue::into_i64).unwrap_or_default(),
                            oninput: make_parse_handler(),
                        }
                    }
                }
                FieldTypeSpec::U64(_) => {
                    rsx! {
                        input {
                            class: "bt-form-input",
                            r#type: "text",
                            inputmode: "numeric",
                            value: o_val.map(ParameterValue::into_u64).unwrap_or_default(),
                            oninput: make_parse_handler(),
                        }
                    }
                }
                FieldTypeSpec::F64(_) => {
                    rsx! {
                        input {
                            class: "bt-form-input",
                            r#type: "text",
                            inputmode: "decimal",
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
