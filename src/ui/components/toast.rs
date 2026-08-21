use gpui::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub created_at: std::time::Instant,
    pub duration_secs: f32,
}

impl Toast {
    pub fn new(message: impl Into<String>, kind: ToastKind) -> Self {
        Self {
            message: message.into(),
            kind,
            created_at: std::time::Instant::now(),
            duration_secs: 3.0,
        }
    }

    pub fn success(msg: impl Into<String>) -> Self {
        Self::new(msg, ToastKind::Success)
    }

    pub fn info(msg: impl Into<String>) -> Self {
        Self::new(msg, ToastKind::Info)
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::new(msg, ToastKind::Error)
    }
}

pub fn render_toast(toast: &Toast) -> impl IntoElement {
    let (bg_color, border_color, icon) = match toast.kind {
        ToastKind::Success => (
            hsla(145.0 / 360.0, 0.4, 0.15, 0.95),
            hsla(145.0 / 360.0, 0.6, 0.45, 1.0),
            "✅",
        ),
        ToastKind::Info => (
            hsla(215.0 / 360.0, 0.4, 0.15, 0.95),
            hsla(215.0 / 360.0, 0.6, 0.50, 1.0),
            "ℹ️",
        ),
        ToastKind::Warning => (
            hsla(40.0 / 360.0, 0.4, 0.15, 0.95),
            hsla(40.0 / 360.0, 0.7, 0.50, 1.0),
            "⚠️",
        ),
        ToastKind::Error => (
            hsla(0.0, 0.4, 0.15, 0.95),
            hsla(0.0, 0.7, 0.50, 1.0),
            "❌",
        ),
    };

    div()
        .flex()
        .items_center()
        .gap_2()
        .px_4()
        .py_2()
        .rounded_lg()
        .bg(bg_color)
        .border_1()
        .border_color(border_color)
        .shadow_lg()
        .text_sm()
        .text_color(rgb(0xffffff))
        .child(icon)
        .child(toast.message.clone())
}
