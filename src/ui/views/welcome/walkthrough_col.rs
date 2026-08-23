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
                .text_color(rgb(0xF1F5F9))
                .child(state.t("welcome.walkthrough_section")),
        )
        .child(render_guide_card(
            "⭐",
            state.t("welcome.guide_basics"),
            state.t("welcome.guide_basics_desc"),
            rgb(0xF59E0B),
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
            rgb(0x00E5BA),
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
            rgb(0x00C4FF),
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
            rgb(0x10B981),
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
    accent_color: Rgba,
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
        .bg(rgb(0x1E2330)) // 浅层蓝灰 Card / Hover
        .border_1()
        .border_color(rgb(0x2D3548))
        .hover(|s| s.bg(rgb(0x252C3D)).border_color(rgb(0x00E5BA))) // 遗迹青蓝 Hover
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
                .bg(rgb(0x161922))
                .border_1()
                .border_color(rgb(0x232838))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(16.0))
                .text_color(accent_color)
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
                        .text_color(rgb(0xF1F5F9)) // 亮白灰 Text Primary
                        .child(title_str),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(rgb(0x94A3B8)) // 蓝灰白 Text Secondary
                        .child(desc_str),
                ),
        )
}
