use dioxus::prelude::*;

use crate::ui::handler::handlers;

handlers!(on_project: (),
          on_valid_tree: ()
);

#[component]
pub fn ExportProject() -> Element {
    let handlers = use_context::<Handlers>();
    rsx! {
        button { onclick: move |_| { handlers.on_project.call(()) },
            "Export project"
        }
    }
}

#[component]
pub fn ExportValidTree() -> Element {
    let handlers = use_context::<Handlers>();

    rsx! {
        button { onclick: move |_| { handlers.on_valid_tree.call(()) },
            "Export valid tree"
        }
    }
}
