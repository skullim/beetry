use std::collections::BTreeMap;

use anyhow::bail;
use beetry_editor_types::{NodeSpec, NodeSpecKey};
use beetry_plugin_types::parameter::{Definition, Type};
use beetry_reconstruction_types::parameter::Parameters;
use dioxus::prelude::*;
use dioxus_logger::tracing::debug;
use serde_value::Value;

use crate::definitions::Point;

#[derive(Debug, Clone)]
pub struct Handlers {
    //@todo ideally it should be only Parameters that are passed
    // right now if node has params then it is created by the on_confirm handler.
    // However this component should not be responsible for that, but rather only for inserting valid params into repository
    pub(crate) on_confirm: EventHandler<(NodeSpecKey, Parameters)>,
    pub(crate) on_cancel: EventHandler<()>,
}

impl Handlers {
    pub(crate) fn new(
        on_confirm: impl FnMut((NodeSpecKey, Parameters)) + 'static,
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
        spec: NodeSpecKey,
    },
}

#[derive(Debug, Props, Clone, PartialEq, Eq)]
pub struct DialogProps {
    state: Signal<State>,
}

#[component]
pub fn Dialog(props: DialogProps) -> Element {
    rsx!()
    // debug!("rendering parameter dialog");
    // let state_read = props.state.read();

    // let (position, spec) = match state_read.clone() {
    //     State::Idle => return rsx! {},
    //     State::Visible { position, spec } => (position, spec),
    // };

    // let parameter_values = use_signal(|| {
    //     let mut values_map = BTreeMap::new();
    //     for param_def in &spec.params_schema.defs {
    //         let default_value = match &param_def.ty {
    //             Type::Boolean => Value::Bool(false),
    //             Type::Integer { bounds: _ } => Value::I64(0),
    //             Type::Float { bounds: _ } => Value::F64(0.0),
    //             Type::String { max_length: _ } => Value::String(String::new()),
    //         };
    //         values_map.insert(param_def.name.clone(), default_value);
    //     }
    //     values_map
    // });

    // let param_defs = spec.params_schema.defs.clone();
    // let has_validation_errors = use_memo(move || {
    //     let values = parameter_values.read();

    //     for param_def in &param_defs {
    //         if validate_parameter(param_def, values.get(&param_def.name)).is_err() {
    //             return true;
    //         }
    //     }
    //     false
    // });

    // let handlers = use_context::<Handlers>();

    // let spec_for_confirm = spec.clone();
    // let on_confirm = move |_| {
    //     if !has_validation_errors() {
    //         let values = parameter_values.read();
    //         let value_iter = values.iter().map(|(k, v)| (k.to_string(), v.clone()));
    //         let serialized_params = Parameters::from_iter(value_iter);
    //         handlers
    //             .on_confirm
    //             .call((spec_for_confirm.clone(), serialized_params));
    //     }
    // };

    // let on_cancel = move |_| {
    //     handlers.on_cancel.call(());
    // };

    // rsx! {
    //     div {
    //         position: "fixed",
    //         top: "0",
    //         left: "0",
    //         width: "100vw",
    //         height: "100vh",
    //         background: "rgba(0,0,0,0.5)",
    //         z_index: "1000",
    //         onclick: on_cancel,

    //         div {
    //             position: "absolute",
    //             left: "{position.x}px",
    //             top: "{position.y}px",
    //             background: "white",
    //             border: "1px solid #ccc",
    //             border_radius: "8px",
    //             box_shadow: "0 4px 16px rgba(0,0,0,0.3)",
    //             min_width: "320px",
    //             max_width: "500px",
    //             padding: "20px",
    //             z_index: "1001",
    //             onclick: move |evt| evt.stop_propagation(),

    //             h3 { margin: "0 0 16px 0", "Configure Parameters for {spec.name()}" }

    //             for param_def in &spec.params_schema.defs {
    //                 div { margin_bottom: "16px",
    //                     ParameterField {
    //                         definition: param_def.clone(),
    //                         values: parameter_values,
    //                     }
    //                 }
    //             }

    //             div {
    //                 display: "flex",
    //                 justify_content: "flex-end",
    //                 gap: "8px",
    //                 margin_top: "20px",

    //                 button {
    //                     padding: "8px 16px",
    //                     border: "1px solid #ddd",
    //                     border_radius: "4px",
    //                     background: "white",
    //                     cursor: "pointer",
    //                     onclick: on_cancel,
    //                     "Cancel"
    //                 }

    //                 button {
    //                     padding: "8px 16px",
    //                     border: if has_validation_errors() { "1px solid #ccc" } else { "1px solid #007acc" },
    //                     border_radius: "4px",
    //                     background: if has_validation_errors() { "#ccc" } else { "#007acc" },
    //                     color: "white",
    //                     cursor: if has_validation_errors() { "not-allowed" } else { "pointer" },
    //                     onclick: on_confirm,
    //                     "Confirm"
    //                 }
    //             }
    //         }
    //     }
    // }
}

#[derive(Props, Clone, PartialEq)]
struct ParameterFieldProps {
    definition: Definition,
    values: Signal<BTreeMap<String, Value>>,
}

#[component]
fn ParameterField(props: ParameterFieldProps) -> Element {
    let def = props.definition.clone();
    let mut values = props.values;

    let def_for_validation = def.clone();
    let validation_error = move || {
        let current_values = values.peek();
        validate_parameter(
            &def_for_validation,
            current_values.get(&def_for_validation.name),
        )
    };

    rsx! {
        label { display: "block", margin_bottom: "4px", font_weight: "bold", {def.name.clone()} }

        if let Some(desc) = &def.description {
            div { font_size: "12px", color: "#666", margin_bottom: "4px", {desc.clone()} }
        }

        match &def.ty {
            Type::Boolean => {
                let def_name = def.name.clone();
                rsx! {
                    input {
                        r#type: "checkbox",
                        checked: false, //values.read().get(&def.name).and_then(|v| v.as_bool()).unwrap_or(false),
                        onchange: move |evt| {
                            let mut vals = values.write();
                            vals.insert(def_name.clone(), Value::Bool(evt.checked()));
                        },
                    }
                }
            }
            Type::Integer { bounds } => {
                let def_name = def.name.clone();
                let error = validation_error();
                let border_color = if error.is_err() { "#ff0000" } else { "#ddd" };
                rsx! {
                    input {
                        r#type: "number",
                        min: bounds.as_ref().map(|b| b.min().to_string()),
                        max: bounds.as_ref().map(|b| b.max().to_string()),
                        value: 0i64, //values.read().get(&def.name).and_then(|v| v.as_i64()).unwrap_or(0).to_string(),
                        width: "100%",
                        padding: "4px 8px",
                        border: "1px solid {border_color}",
                        border_radius: "4px",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<i64>() {
                                values.with_mut(|write| write.insert(def_name.clone(), Value::I64(val)));
                            }
                        },
                    }
                    if let Err(error_msg) = error {
                        div { color: "#ff0000", font_size: "12px", margin_top: "4px", {format!("{error_msg}")} }
                    }
                }
            }
            Type::Float { bounds } => {
                let def_name = def.name.clone();
                let error = validation_error();
                let border_color = if error.is_err() { "#ff0000" } else { "#ddd" };
                rsx! {
                    input {
                        r#type: "number",
                        step: "1.00",
                        min: bounds.as_ref().map(|b| b.min().to_string()),
                        max: bounds.as_ref().map(|b| b.max().to_string()),
                        value: 0.0, //values.read().get(&def.name).and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                        width: "100%",
                        padding: "4px 8px",
                        border: "1px solid {border_color}",
                        border_radius: "4px",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<f64>() {
                                values.with_mut(|write| write.insert(def_name.clone(), Value::F64(val)));
                            }
                        },
                    }
                    if let Err(error_msg) = error {
                        div { color: "#ff0000", font_size: "12px", margin_top: "4px", {format!("{error_msg}")} }
                    }
                }
            }
            Type::String { max_length } => {
                let def_name = def.name.clone();
                let error = validation_error();
                let border_color = if error.is_err() { "#ff0000" } else { "#ddd" };
                rsx! {
                    input {
                        r#type: "text",
                        maxlength: max_length.map(|len| len.to_string()),
                        value: "", //values.read().get(&def.name).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        width: "100%",
                        padding: "4px 8px",
                        border: "1px solid {border_color}",
                        border_radius: "4px",
                        oninput: move |evt| {
                            let value = evt.value();
                            let mut vals = values.write();
                            vals.insert(def_name.clone(), Value::String(value));
                        },
                    }
                    if let Err(error_msg) = error {
                        div { color: "#ff0000", font_size: "12px", margin_top: "4px", {format!("{error_msg}")} }
                    }
                }
            }
        }
    }
}

fn validate_parameter(param: &Definition, value: Option<&Value>) -> anyhow::Result<()> {
    match (&param.ty, value) {
        (Type::Integer { bounds }, Some(Value::I64(n))) => {
            if let Some(bounds) = bounds
                && (*n < bounds.min() || *n > bounds.max())
            {
                bail!(
                    "Value must be between {} and {}",
                    bounds.min(),
                    bounds.max()
                );
            }
            Ok(())
        }
        (Type::Float { bounds }, Some(Value::F64(n))) => {
            if let Some(bounds) = bounds
                && (*n < bounds.min() as f64 || *n > bounds.max() as f64)
            {
                bail!(
                    "Value must be between {} and {}",
                    bounds.min(),
                    bounds.max()
                );
            }
            Ok(())
        }
        (Type::String { max_length }, Some(Value::String(s))) => {
            if let Some(max_len) = max_length
                && s.len() > *max_len
            {
                bail!("String must be at most {} characters", max_len);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
