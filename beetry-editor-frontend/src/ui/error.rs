use core::fmt;
use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct ErrorQueueState(Signal<ErrorQueue>);

impl ErrorQueueState {
    pub(crate) fn new() -> Self {
        Self(Signal::new(ErrorQueue::new()))
    }

    pub fn push(&mut self, err: impl fmt::Display) {
        self.0.with_mut(|e| e.push(err));
    }
}

impl Deref for ErrorQueueState {
    type Target = Signal<ErrorQueue>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ErrorQueueState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLog {
    pub id: u64,
    pub message: String,
    pub timestamp_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorQueue {
    entries: VecDeque<ErrorLog>,
    next_id: u64,
}

impl ErrorQueue {
    const MAX_ENTRIES: usize = 50;

    fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            next_id: 0,
        }
    }

    pub fn push(&mut self, err: impl fmt::Display) {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.entries.push_back(ErrorLog {
            id,
            message: format!("{err}"),
            timestamp_secs: now_unix_secs(),
        });
        if self.entries.len() > Self::MAX_ENTRIES {
            self.entries.pop_front();
        }
    }

    pub fn snapshot(&self) -> VecDeque<ErrorLog> {
        self.entries.clone()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn dismiss(&mut self, id: u64) {
        self.entries.retain(|entry| entry.id != id);
    }
}

#[component]
pub fn Dialog() -> Element {
    let mut queue = use_context::<ErrorQueueState>();
    let items = queue.read().snapshot();
    if items.is_empty() {
        return rsx! {};
    }

    let entries = items.into_iter().rev().map(|item| {
        let mut queue_for_entry = queue;
        let timestamp = format_timestamp(item.timestamp_secs);

        rsx! {
            li { key: "{item.id}", class: "bt-error-item",
                div { class: "bt-error-item-header",
                    div { class: "bt-error-item-meta",
                        span { class: "bt-error-timestamp", "{timestamp}" }
                        button {
                            class: "bt-error-dismiss-btn",
                            onclick: move |_| queue_for_entry.with_mut(|q| q.dismiss(item.id)),
                            "x"
                        }
                    }
                }
                div { class: "bt-error-message", {item.message} }
            }
        }
    });

    rsx! {
        div { class: "bt-error-dialog",
            div { class: "bt-error-header",
                b { class: "bt-error-title", "Errors" }
                button {
                    class: "bt-error-clear-btn",
                    onclick: move |_| queue.with_mut(ErrorQueue::clear),
                    "Clear"
                }
            }
            ul { class: "bt-error-list", {entries} }
        }
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn format_timestamp(epoch_secs: u64) -> String {
    DateTime::<Utc>::from_timestamp(epoch_secs.cast_signed(), 0).map_or_else(
        || String::from("--:--:--"),
        |dt| dt.with_timezone(&Local).format("%H:%M:%S").to_string(),
    )
}
