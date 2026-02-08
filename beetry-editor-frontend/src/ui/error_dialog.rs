use dioxus::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorMsg {
    pub id: u64,
    pub source: &'static str,
    pub message: String,
    pub count: usize,
    pub timestamp_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorMsgQueue {
    entries: Vec<ErrorMsg>,
    next_id: u64,
}

impl ErrorMsgQueue {
    const MAX_ENTRIES: usize = 50;

    pub fn new() -> Self {
        Self {
            entries: vec![],
            next_id: 1,
        }
    }

    pub fn push(&mut self, source: &'static str, message: impl Into<String>) {
        let message = message.into();
        let now = now_unix_secs();
        if let Some(last) = self.entries.last_mut()
            && last.source == source
            && last.message == message
        {
            last.count += 1;
            last.timestamp_secs = now;
            return;
        }

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.entries.push(ErrorMsg {
            id,
            source,
            message: message.clone(),
            count: 1,
            timestamp_secs: now,
        });
        if self.entries.len() > Self::MAX_ENTRIES {
            let to_drop = self.entries.len() - Self::MAX_ENTRIES;
            self.entries.drain(0..to_drop);
        }
    }

    pub fn snapshot(&self) -> Vec<ErrorMsg> {
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
pub fn ErrorDialog() -> Element {
    let mut queue = use_context::<Signal<ErrorMsgQueue>>();
    let items = queue.read().snapshot();
    if items.is_empty() {
        return rsx! {};
    }

    let entries = items.into_iter().rev().map(|item| {
        let mut queue_for_entry = queue;
        let timestamp = format_timestamp(item.timestamp_secs);
        let count_badge = if item.count > 1 {
            rsx! {
                span {
                    style: "font-size: 10px; color: #8b0000; border: 1px solid #efb8b8; background: #fff5f5; border-radius: 999px; padding: 0 6px;",
                    "x{item.count}"
                }
            }
        } else {
            rsx! {}
        };

        rsx! {
            li {
                key: "{item.id}",
                style: "list-style: none; margin-bottom: 8px; padding: 8px; border: 1px solid #efb8b8; border-radius: 8px; background: #fffafa;",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;",
                    div {
                        style: "display: flex; align-items: center; gap: 6px;",
                        span {
                            style: "font-size: 10px; color: #8b0000; border: 1px solid #efb8b8; background: #fff0f0; border-radius: 999px; padding: 0 6px;",
                            "{item.source}"
                        }
                        {count_badge}
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 6px;",
                        span { style: "font-size: 10px; color: #9f5e5e;", "{timestamp}" }
                        button {
                            onclick: move |_| queue_for_entry.with_mut(|q| q.dismiss(item.id)),
                            style: "font-size: 12px; line-height: 1; color: #8b0000; border: 1px solid #efb8b8; background: #fff; border-radius: 4px; width: 18px; height: 18px; cursor: pointer;",
                            "x"
                        }
                    }
                }
                div {
                    style: "font-size: 12px; color: #631f1f; white-space: pre-wrap; word-break: break-word;",
                    {item.message.clone()}
                }
            }
        }
    });

    rsx! {
        div {
            style: "position: fixed; right: 16px; bottom: 16px; width: 420px; max-width: calc(100vw - 24px); z-index: 2000; border: 1px solid #d66; background: #fff; padding: 10px; border-radius: 10px; box-shadow: 0 6px 18px rgba(0,0,0,0.2);",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                b { "Errors ({queue.read().entries.len()})" }
                button {
                    onclick: move |_| queue.with_mut(ErrorMsgQueue::clear),
                    style: "font-size: 12px; border: 1px solid #d66; border-radius: 6px; background: #fff; color: #8b0000; padding: 2px 8px; cursor: pointer;",
                    "Clear"
                }
            }
            ul {
                style: "margin: 0; padding: 0; max-height: 220px; overflow: auto;",
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
    let minutes = (epoch_secs / 60) % 60;
    let hours = (epoch_secs / 3600) % 24;
    let seconds = epoch_secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
