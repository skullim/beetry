use chrono::{DateTime, Local, Utc};
use core::fmt;
use dioxus::prelude::*;
use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy)]
pub struct ErrorQueueState(Signal<ErrorQueue>);

impl ErrorQueueState {
    pub(crate) fn new() -> Self {
        Self(Signal::new(ErrorQueue::new()))
    }
}

impl ErrorQueueState {
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
            li {
                key: "{item.id}",
                style: "list-style: none; margin-bottom: 8px; padding: 8px; border: 1px solid #efb8b8; border-radius: 8px; background: #fffafa;",
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;",
                    div { style: "display: flex; align-items: center; gap: 6px;",
                        span { style: "font-size: 10px; color: #9f5e5e;", "{timestamp}" }
                        button {
                            onclick: move |_| queue_for_entry.with_mut(|q| q.dismiss(item.id)),
                            style: "font-size: 12px; line-height: 1; color: #8b0000; border: 1px solid #efb8b8; background: #fff; border-radius: 4px; width: 18px; height: 18px; cursor: pointer;",
                            "x"
                        }
                    }
                }
                div { style: "font-size: 12px; color: #631f1f; white-space: pre-wrap; word-break: break-word;",
                    {item.message}
                }
            }
        }
    });

    rsx! {
        div { style: "position: fixed; right: 16px; bottom: 16px; width: 420px; max-width: calc(100vw - 24px); z-index: 2000; border: 1px solid #d66; background: #fff; padding: 10px; border-radius: 10px; box-shadow: 0 6px 18px rgba(0,0,0,0.2);",
            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                b { "Errors" }
                button {
                    onclick: move |_| queue.with_mut(ErrorQueue::clear),
                    style: "font-size: 12px; border: 1px solid #d66; border-radius: 6px; background: #fff; color: #8b0000; padding: 2px 8px; cursor: pointer;",
                    "Clear"
                }
            }
            ul { style: "margin: 0; padding: 0; max-height: 220px; overflow: auto;",
                {entries}
            }
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
    DateTime::<Utc>::from_timestamp(epoch_secs as i64, 0)
        .map(|dt| dt.with_timezone(&Local).format("%H:%M:%S").to_string())
        .unwrap_or_else(|| String::from("--:--:--"))
}
