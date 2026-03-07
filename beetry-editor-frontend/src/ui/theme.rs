use dioxus::prelude::*;

const THEME_CSS: &str = "
    html, body {
        margin: 0;
        padding: 0;
        height: 100%;
        overflow: hidden;
    }

    :root {
        --bt-font-family: system-ui, -apple-system, sans-serif;
        --bt-font-sm: 10px;
        --bt-font-md: 12px;
        --bt-font-lg: 14px;
        --bt-bg: #eef2f7;
        --bt-bg-soft: #e4ebf3;
        --bt-panel: #f1f6fc;
        --bt-panel-strong: #f8fbff;
        --bt-surface: #ffffff;
        --bt-surface-soft: #f7fbff;
        --bt-surface-muted: #edf4fc;
        --bt-border: #ccd8e6;
        --bt-border-soft: #c7d5e6;
        --bt-border-soft-hover: #a7bbd6;
        --bt-border-focus: #8fb8e0;
        --bt-border-accent: #8ea9ca;
        --bt-text: #0f172a;
        --bt-text-soft: #55637a;
        --bt-text-strong-soft: #2a4365;
        --bt-text-muted: #274060;
        --bt-accent: #0f766e;
        --bt-accent-strong: #0b5f58;
        --bt-accent-button: #119c90;
        --bt-accent-button-hover: #0f8a7f;
        --bt-accent-soft: #d6f2ef;
        --bt-danger: #c2363f;
        --bt-focus-ring: rgba(143, 184, 224, 0.2);
        --bt-shadow: 0 14px 36px rgba(15, 23, 42, 0.12);
        --bt-radius-lg: 16px;
        --bt-radius-md: 10px;
        --bt-radius-sm: 8px;
        --bt-hover-lift: 1px;
        --bt-sidebar-list-gap: 6px;
        --bt-sidebar-item-height: 34px;
        --bt-menu-fill: #ffffff;
        --bt-menu-stroke: #cccccc;
        --bt-menu-text: #111111;
    }

    .bt-editor-shell {
        height: 100vh;
        box-sizing: border-box;
        padding: 18px;
        background:
            radial-gradient(1300px 700px at 0% 0%, rgba(255,255,255,0.95), rgba(238,242,247,0.9)),
            linear-gradient(135deg, var(--bt-bg), var(--bt-bg-soft));
        color: var(--bt-text);
        font-family: var(--bt-font-family);
    }

    .bt-editor-shell button,
    .bt-editor-shell input,
    .bt-editor-shell select,
    .bt-editor-shell textarea {
        font-family: var(--bt-font-family);
    }

    .bt-editor-grid {
        display: grid;
        grid-template-columns: minmax(260px, 320px) minmax(720px, 1fr) minmax(180px, 240px);
        gap: 14px;
        height: 100%;
        min-height: 0;
        align-items: stretch;
    }

    .bt-panel {
        background: linear-gradient(180deg, var(--bt-panel), var(--bt-panel-strong));
        border: 1px solid var(--bt-border);
        border-radius: var(--bt-radius-lg);
        box-shadow: var(--bt-shadow);
        padding: 14px;
        min-height: 0;
    }

    .bt-panel-title {
        margin: 0 0 10px 0;
        font-size: var(--bt-font-lg);
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
        border: 1px solid var(--bt-border-soft);
        border-radius: var(--bt-radius-sm);
        background: var(--bt-surface-soft);
        color: var(--bt-text);
        padding: 9px 10px;
        font-size: var(--bt-font-md);
        outline: none;
    }

    .bt-sidebar-search:focus {
        border-color: var(--bt-border-focus);
        box-shadow: 0 0 0 3px var(--bt-focus-ring);
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
        font-size: var(--bt-font-md);
        font-weight: 700;
        letter-spacing: 0.04em;
        color: var(--bt-text-soft);
    }

    .bt-sidebar-list {
        display: grid;
        gap: var(--bt-sidebar-list-gap);
        max-height: calc(
            (3 * var(--bt-sidebar-item-height)) + (2 * var(--bt-sidebar-list-gap)) + (2 * var(--bt-hover-lift))
        );
        overflow: auto;
        padding-top: var(--bt-hover-lift);
        padding-bottom: var(--bt-hover-lift);
        padding-right: 2px;
    }

    .bt-sidebar-empty {
        margin: 0;
        padding: 6px 2px;
        font-size: var(--bt-font-md);
        color: #6b7d93;
    }

    .bt-btn {
        border: 1px solid transparent;
        border-radius: var(--bt-radius-sm);
        background: #f5f9ff;
        color: var(--bt-text);
        font-size: var(--bt-font-md);
        font-weight: 600;
        line-height: 1.1;
        padding: 8px 11px;
        cursor: pointer;
        transition: background 130ms ease, border-color 130ms ease, transform 130ms ease, box-shadow 130ms ease;
    }

    .bt-btn:hover {
        transform: translateY(calc(-1 * var(--bt-hover-lift)));
        box-shadow: 0 6px 14px rgba(15, 23, 42, 0.08);
    }

    .bt-btn:active {
        transform: translateY(0);
        box-shadow: none;
    }

    .bt-btn--sidebar {
        width: 100%;
        min-height: var(--bt-sidebar-item-height);
        box-sizing: border-box;
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

    .bt-btn--dialog-secondary {
        border-color: var(--bt-border-soft);
        background: var(--bt-surface-soft);
        color: var(--bt-text-muted);
    }

    .bt-btn--dialog-secondary:hover {
        border-color: var(--bt-border-soft-hover);
        background: var(--bt-surface-muted);
    }

    .bt-btn--dialog-primary {
        border-color: #0e6f67;
        background: var(--bt-accent-button);
        color: #ffffff;
    }

    .bt-btn--dialog-primary:hover {
        border-color: #0b5f58;
        background: var(--bt-accent-button-hover);
    }

    .bt-dialog-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: rgba(15, 23, 42, 0.42);
        z-index: 1000;
    }

    .bt-dialog {
        position: absolute;
        background: linear-gradient(180deg, #ffffff, #f8fbff);
        border: 1px solid var(--bt-border);
        border-radius: var(--bt-radius-md);
        box-shadow: var(--bt-shadow);
        padding: 18px;
        z-index: 1001;
    }

    .bt-dialog--parameter {
        min-width: 320px;
        max-width: 500px;
    }

    .bt-dialog--channel {
        min-width: 320px;
        max-width: 460px;
    }

    .bt-dialog-title {
        margin: 0 0 14px 0;
        font-size: var(--bt-font-lg);
        font-weight: 700;
        line-height: 1.3;
        color: var(--bt-text);
    }

    .bt-dialog-subtitle {
        font-size: var(--bt-font-md);
        color: var(--bt-text-soft);
        font-weight: 600;
    }

    .bt-dialog-actions {
        display: flex;
        justify-content: flex-end;
        gap: 8px;
        margin-top: 20px;
    }

    .bt-form-field {
        margin-bottom: 16px;
    }

    .bt-form-label {
        display: block;
        margin-bottom: 4px;
        font-size: var(--bt-font-md);
        font-weight: 700;
        color: var(--bt-text);
    }

    .bt-form-description {
        margin-bottom: 6px;
        font-size: var(--bt-font-md);
        color: var(--bt-text-soft);
    }

    .bt-form-input {
        width: 100%;
        box-sizing: border-box;
        border: 1px solid var(--bt-border-soft);
        border-radius: var(--bt-radius-sm);
        background: var(--bt-surface);
        color: var(--bt-text);
        padding: 6px 8px;
        font-size: var(--bt-font-md);
        outline: none;
    }

    .bt-form-input:focus {
        border-color: var(--bt-border-focus);
        box-shadow: 0 0 0 3px var(--bt-focus-ring);
    }

    .bt-form-checkbox {
        width: 16px;
        height: 16px;
        accent-color: #0f766e;
    }

    .bt-form-error {
        margin: 4px 0 0 0;
        color: var(--bt-danger);
        font-size: var(--bt-font-md);
        font-weight: 600;
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
        min-height: 0;
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
        border: 1px solid var(--bt-border-accent);
        background: rgba(255, 255, 255, 0.92);
        color: var(--bt-text-strong-soft);
        font-size: var(--bt-font-lg);
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
        border: 1px solid var(--bt-border-accent);
        background: rgba(255, 255, 255, 0.96);
        color: var(--bt-text-strong-soft);
        font-size: var(--bt-font-md);
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
        height: calc(100vh - 140px);
        min-height: 0;
    }

    .bt-workspace-canvas svg {
        font-family: var(--bt-font-family);
    }

    .bt-workspace-canvas svg text {
        font-size: var(--bt-font-md);
        user-select: none;
        -webkit-user-select: none;
    }

    .bt-workspace-canvas svg .bt-text-sm {
        font-size: var(--bt-font-sm);
    }

    .bt-topbar-chip {
        display: inline-block;
        margin: 0 2px;
        padding: 1px 6px;
        border-radius: 999px;
        border: 1px solid var(--bt-border-soft-hover);
        background: var(--bt-surface-muted);
        color: #1f3a5c;
        font-size: var(--bt-font-md);
        font-weight: 700;
        line-height: 1.2;
        vertical-align: baseline;
    }

    .bt-error-dialog {
        position: fixed;
        right: 16px;
        bottom: 16px;
        width: 420px;
        max-width: calc(100vw - 24px);
        z-index: 2000;
        border: 1px solid #d66;
        background: var(--bt-surface);
        padding: 10px;
        border-radius: var(--bt-radius-md);
        box-shadow: 0 6px 18px rgba(0, 0, 0, 0.2);
    }

    .bt-error-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
    }

    .bt-error-title {
        color: #6e2525;
    }

    .bt-error-clear-btn {
        font-size: var(--bt-font-md);
        border: 1px solid #d66;
        border-radius: 6px;
        background: var(--bt-surface);
        color: #8b0000;
        padding: 2px 8px;
        cursor: pointer;
    }

    .bt-error-list {
        margin: 0;
        padding: 0;
        max-height: 220px;
        overflow: auto;
    }

    .bt-error-item {
        list-style: none;
        margin-bottom: 8px;
        padding: 8px;
        border: 1px solid #efb8b8;
        border-radius: var(--bt-radius-sm);
        background: #fffafa;
    }

    .bt-error-item-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 6px;
    }

    .bt-error-item-meta {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .bt-error-timestamp {
        font-size: var(--bt-font-md);
        color: #9f5e5e;
    }

    .bt-error-dismiss-btn {
        font-size: var(--bt-font-md);
        line-height: 1;
        color: #8b0000;
        border: 1px solid #efb8b8;
        background: var(--bt-surface);
        border-radius: 4px;
        width: 18px;
        height: 18px;
        padding: 0;
        display: grid;
        place-items: center;
        cursor: pointer;
    }

    .bt-error-message {
        font-size: var(--bt-font-md);
        color: #631f1f;
        white-space: pre-wrap;
        word-break: break-word;
    }
";

#[component]
pub(crate) fn GlobalStyle() -> Element {
    rsx! {
        style { {THEME_CSS} }
    }
}
