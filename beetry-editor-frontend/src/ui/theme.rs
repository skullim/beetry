use dioxus::prelude::*;

const THEME_CSS: &str = r#"
    :root {
        --bt-bg: #eef2f7;
        --bt-bg-soft: #e4ebf3;
        --bt-panel: #f1f6fc;
        --bt-panel-strong: #f8fbff;
        --bt-border: #ccd8e6;
        --bt-text: #0f172a;
        --bt-text-soft: #55637a;
        --bt-accent: #0f766e;
        --bt-accent-strong: #0b5f58;
        --bt-accent-soft: #d6f2ef;
        --bt-shadow: 0 14px 36px rgba(15, 23, 42, 0.12);
        --bt-radius-lg: 16px;
        --bt-radius-md: 10px;
        --bt-radius-sm: 8px;
    }

    .bt-editor-shell {
        min-height: 100vh;
        box-sizing: border-box;
        padding: 18px;
        background:
            radial-gradient(1300px 700px at 0% 0%, rgba(255,255,255,0.95), rgba(238,242,247,0.9)),
            linear-gradient(135deg, var(--bt-bg), var(--bt-bg-soft));
        color: var(--bt-text);
        font-family: system-ui, -apple-system, sans-serif;
    }

    .bt-editor-grid {
        display: grid;
        grid-template-columns: minmax(260px, 320px) minmax(720px, 1fr) minmax(180px, 240px);
        gap: 14px;
        align-items: start;
    }

    .bt-panel {
        background: linear-gradient(180deg, var(--bt-panel), var(--bt-panel-strong));
        border: 1px solid var(--bt-border);
        border-radius: var(--bt-radius-lg);
        box-shadow: var(--bt-shadow);
        padding: 14px;
    }

    .bt-panel-title {
        margin: 0 0 10px 0;
        font-size: 13px;
        font-weight: 700;
        letter-spacing: 0.04em;
        color: var(--bt-text-soft);
    }

    .bt-sidebar {
        display: grid;
        gap: 12px;
    }

    .bt-sidebar-search {
        width: 100%;
        box-sizing: border-box;
        border: 1px solid #c7d5e6;
        border-radius: var(--bt-radius-sm);
        background: #f7fbff;
        color: var(--bt-text);
        padding: 9px 10px;
        font-size: 12px;
        outline: none;
    }

    .bt-sidebar-search:focus {
        border-color: #8fb8e0;
        box-shadow: 0 0 0 3px rgba(143, 184, 224, 0.2);
    }

    .bt-sidebar-section {
        display: grid;
        gap: 8px;
        padding: 10px;
        border-radius: var(--bt-radius-md);
        border: 1px solid #dbe6f2;
        background: rgba(244, 249, 255, 0.86);
    }

    .bt-sidebar-section h3 {
        margin: 0;
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.04em;
        color: var(--bt-text-soft);
    }

    .bt-sidebar-list {
        display: grid;
        gap: 6px;
        max-height: 160px;
        overflow: auto;
        padding-top: 2px;
        padding-right: 2px;
    }

    .bt-sidebar-empty {
        margin: 0;
        padding: 6px 2px;
        font-size: 12px;
        color: #6b7d93;
    }

    .bt-btn {
        border: 1px solid transparent;
        border-radius: var(--bt-radius-sm);
        background: #f5f9ff;
        color: var(--bt-text);
        font-size: 12px;
        font-weight: 600;
        line-height: 1.1;
        padding: 8px 11px;
        cursor: pointer;
        transition: background 130ms ease, border-color 130ms ease, transform 130ms ease, box-shadow 130ms ease;
    }

    .bt-btn:hover {
        transform: translateY(-1px);
        box-shadow: 0 6px 14px rgba(15, 23, 42, 0.08);
    }

    .bt-btn:active {
        transform: translateY(0);
        box-shadow: none;
    }

    .bt-btn--sidebar {
        width: 100%;
        text-align: left;
        border-color: #d4e0ee;
        background: #edf4fc;
    }

    .bt-btn--sidebar:hover {
        border-color: #bdd0e5;
        background: #e6eff9;
    }

    .bt-btn--toolbar {
        width: 100%;
        text-align: left;
        border-color: #9fc9c3;
        background: #c4e7e2;
        color: #0c5650;
    }

    .bt-btn--toolbar:hover {
        background: #b7dfd9;
        border-color: #86b8b0;
    }

    .bt-toolbar-actions {
        display: grid;
        gap: 8px;
    }

    .bt-workspace-shell {
        display: grid;
        gap: 10px;
        background: linear-gradient(180deg, #f5f9ff, #eef4fb);
        border-color: #c3d4e9;
    }

    .bt-topbar {
        position: relative;
        display: flex;
        justify-content: flex-end;
        align-items: center;
        min-height: 24px;
        z-index: 4;
        overflow: visible;
    }


    .bt-topbar-help {
        width: 24px;
        height: 24px;
        border-radius: 999px;
        border: 1px solid #8ea9ca;
        background: rgba(255, 255, 255, 0.92);
        color: #2a4365;
        font-size: 13px;
        font-weight: 700;
        display: grid;
        place-items: center;
        cursor: help;
        user-select: none;
    }

    .bt-topbar-help-tooltip {
        position: absolute;
        right: 0;
        top: calc(100% + 6px);
        z-index: 5;
        padding: 4px 8px;
        border-radius: 6px;
        border: 1px solid #8ea9ca;
        background: rgba(255, 255, 255, 0.96);
        color: #2a4365;
        font-size: 12px;
        font-weight: 600;
        white-space: normal;
        pointer-events: none;
    }

    .bt-topbar-help-tooltip p {
        margin: 0;
    }

    .bt-workspace-canvas {
        overflow: auto;
        border: 1px solid #3c495e;
        border-radius: var(--bt-radius-md);
        background: radial-gradient(700px 520px at 14% 12%, #25385d, #13203a 68%);
        box-shadow: inset 0 0 0 1px rgba(173, 198, 237, 0.08);
        width: 100%;
        min-height: 760px;
        max-height: calc(100vh - 140px);
    }

    .bt-input-chip {
        display: inline-block;
        margin: 0 2px;
        padding: 1px 6px;
        border-radius: 999px;
        border: 1px solid #a7bbd6;
        background: #edf4fc;
        color: #1f3a5c;
        font-size: 11px;
        font-weight: 700;
        line-height: 1.2;
        vertical-align: baseline;
    }
"#;

#[component]
pub(crate) fn GlobalStyle() -> Element {
    rsx! {
        style { {THEME_CSS} }
    }
}
