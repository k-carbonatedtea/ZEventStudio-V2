use gpui::*;
use crate::ui::state::{AppState, AppWorkspaceTab};

/// 渲染 VS Code 风格的右侧演练与核心功能概览列
pub fn render_walkthrough_section(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(14.0))
                .text_color(hsla(0.0, 0.0, 1.0, 0.90))
                .child(state.t("welcome.walkthrough_section")),
        )
        .child(render_guide_card(
            "⭐",
            state.t("welcome.guide_basics"),
            state.t("welcome.guide_basics_desc"),
            hsla(45.0 / 360.0, 0.70, 0.30, 0.8),
            {
                let model = model.clone();
                move |cx| {
                    model.update(cx, |st, cx| {
                        st.current_tab = AppWorkspaceTab::Flowchart;
                        cx.notify();
                    });
                }
            },
        ))
        .child(render_guide_card(
            "💡",
            state.t("welcome.guide_evfl"),
            state.t("welcome.guide_evfl_desc"),
            hsla(200.0 / 360.0, 0.60, 0.25, 0.8),
            {
                let model = model.clone();
                move |cx| {
                    model.update(cx, |st, cx| {
                        st.current_tab = AppWorkspaceTab::Flowchart;
                        st.sidebar_open = true;
                        cx.notify();
                    });
                }
            },
        ))
        .child(render_guide_card(
            "🎭",
            state.t("welcome.guide_actors"),
            state.t("welcome.guide_actors_desc"),
            hsla(280.0 / 360.0, 0.50, 0.25, 0.8),
            {
                let model = model.clone();
                move |cx| {
                    model.update(cx, |st, cx| {
                        st.current_tab = AppWorkspaceTab::Actors;
                        cx.notify();
                    });
                }
            },
        ))
        .child(render_guide_card(
            "💬",
            state.t("welcome.guide_msbt"),
            state.t("welcome.guide_msbt_desc"),
            hsla(145.0 / 360.0, 0.50, 0.25, 0.8),
            {
                let model = model.clone();
                move |cx| {
                    model.update(cx, |st, cx| {
                        st.current_tab = AppWorkspaceTab::Events;
                        cx.notify();
                    });
                }
            },
        ))
}

fn render_guide_card<F>(
    icon_str: &'static str,
    title: &str,
    desc: &str,
    bg_color: Hsla,
    on_click: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let title_str = title.to_string();
    let desc_str = desc.to_string();

    div()
        .px_3p5()
        .py_3()
        .rounded_lg()
        .bg(bg_color)
        .border_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.22, 1.0))
        .hover(|s| s.border_color(hsla(215.0 / 360.0, 0.60, 0.45, 1.0)))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap_3()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(
            div()
                .w(px(32.0))
                .h(px(32.0))
                .rounded_md()
                .bg(hsla(220.0 / 360.0, 0.40, 0.12, 1.0))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(16.0))
                .child(icon_str),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.5))
                        .text_color(hsla(0.0, 0.0, 1.0, 0.95))
                        .child(title_str),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(hsla(215.0 / 360.0, 0.20, 0.65, 1.0))
                        .child(desc_str),
                ),
        )
}
