use gpui::*;
use crate::ui::state::{AppState, AppWorkspaceTab};

/// 渲染 VS Code 风格的右侧演练与核心功能概览列
pub fn render_walkthrough_section(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(14.0))
                .text_color(pal.text_primary)
                .child(state.t("welcome.walkthrough_section")),
        )
        .child(render_guide_card(
            "⭐",
            state.t("welcome.guide_basics"),
            state.t("welcome.guide_basics_desc"),
            pal.secondary_accent,
            state,
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
            pal.primary_accent,
            state,
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
            pal.primary_accent,
            state,
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
            pal.secondary_accent,
            state,
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
    state: &AppState,
    on_click: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let pal = state.theme();
    let title_str = title.to_string();
    let desc_str = desc.to_string();

    div()
        .px_3p5()
        .py_3()
        .rounded_lg()
        .bg(pal.card_surface) // 浅层蓝灰 Card / Hover
        .border_1()
        .border_color(pal.card_border)
        .hover(move |s| s.bg(pal.card_hover).border_color(pal.primary_accent)) // 遗迹青蓝 Hover
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
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border)
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
                        .text_color(pal.text_primary) // 亮白灰 Text Primary
                        .child(title_str),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary) // 蓝灰白 Text Secondary
                        .child(desc_str),
                ),
        )
}
