use gpui::*;
use crate::ui::state::AppState;
/// 渲染内存中未保存的文档修改警告条目
pub fn render_unsaved_warning_banner(
    state: &AppState,
    count: usize,
    model_save_all: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let m = model_save_all.clone();

    div()
        .px_3()
        .py_2()
        .bg(if pal.is_dark { rgb(0x282315) } else { rgb(0xFFF9E6) })
        .border_b_1()
        .border_color(pal.secondary_accent)
        .flex()
        .items_center()
        .justify_between()
        .flex_shrink_0()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(pal.secondary_accent)
                        .child("⚠️"),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(pal.secondary_accent)
                        .child(format!("{} ({} 项)", state.t("source_control.unsaved_tabs_hint"), count)),
                ),
        )
        .child(
            div()
                .px_2()
                .py_0p5()
                .rounded_xs()
                .bg(pal.secondary_accent)
                .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .cursor_pointer()
                .hover(|s| s.opacity(0.85))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    m.update(cx, |st, cx| {
                        st.save_all_modified_documents();
                        st.refresh_git_status();
                        cx.notify();
                    });
                })
                .child(state.t("source_control.save_all_tabs")),
        )
}

/// 渲染工作区完全干净视图
#[allow(dead_code)]
pub fn render_clean_working_tree_view(
    state: &AppState,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    div()
        .p_5()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .text_center()
        .child(
            div()
                .text_size(px(26.0))
                .text_color(pal.primary_accent)
                .child("✔"),
        )
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(pal.text_primary)
                .child(state.t("source_control.clean_tree")),
        )
        .child(
            div()
                .text_size(px(10.5))
                .text_color(pal.text_secondary)
                .child(state.t("source_control.all_saved_hint")),
        )
}

/// 渲染暂存分区的表头行 (高度恒定 30px)
pub fn render_staged_header(
    count: usize,
    is_collapsed: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_toggle = model.clone();
    let model_unstage_all = model.clone();

    div()
        .h(px(30.0))
        .w_full()
        .px_2()
        .bg(pal.card_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_toggle.update(cx, |st, cx| {
                st.git_staged_collapsed = !st.git_staged_collapsed;
                cx.notify();
            });
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(if is_collapsed { "▶" } else { "▼" }),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .child("暂存的更改"),
                )
                .child(
                    div()
                        .min_w(px(14.0))
                        .h(px(14.0))
                        .px_1()
                        .rounded_full()
                        .bg(pal.primary_accent)
                        .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                        .text_size(px(8.5))
                        .font_weight(FontWeight::BOLD)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(count.to_string()),
                ),
        )
        // 取消暂存全部按钮 (-)
        .child(
            div()
                .px_1p5()
                .py_0p5()
                .rounded_xs()
                .cursor_pointer()
                .text_size(px(12.0))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
                .hover(|s| s.bg(pal.card_hover).text_color(pal.secondary_accent))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_unstage_all.update(cx, |st, cx| {
                        st.git_unstage_all();
                        cx.notify();
                    });
                })
                .child("-"),
        )
}

/// 渲染未暂存更改分区的表头行 (高度恒定 30px)
pub fn render_unstaged_header(
    count: usize,
    is_collapsed: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_toggle = model.clone();
    let model_stage_all = model.clone();
    let model_discard_all = model.clone();

    div()
        .h(px(30.0))
        .w_full()
        .px_2()
        .bg(pal.card_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_toggle.update(cx, |st, cx| {
                st.git_unstaged_collapsed = !st.git_unstaged_collapsed;
                cx.notify();
            });
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(if is_collapsed { "▶" } else { "▼" }),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .child("更改"),
                )
                .child(
                    div()
                        .min_w(px(14.0))
                        .h(px(14.0))
                        .px_1()
                        .rounded_full()
                        .bg(pal.secondary_accent)
                        .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                        .text_size(px(8.5))
                        .font_weight(FontWeight::BOLD)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(count.to_string()),
                ),
        )
        // 操作按钮组: 全部放弃 (↩) 与 全部暂存 (+)
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_xs()
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.secondary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_discard_all.update(cx, |st, cx| {
                                st.git_discard_all();
                                cx.notify();
                            });
                        })
                        .child("↩"),
                )
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_xs()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.primary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_stage_all.update(cx, |st, cx| {
                                st.git_stage_all();
                                cx.notify();
                            });
                        })
                        .child("+"),
                ),
        )
}


/// 渲染紧凑的工作区干净横条
pub fn render_compact_clean_tree_banner(
    state: &AppState,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    div()
        .h(px(32.0))
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .bg(pal.card_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex_shrink_0()
        .child(
            div()
                .text_size(px(12.0))
                .text_color(pal.primary_accent)
                .child("✔"),
        )
        .child(
            div()
                .text_size(px(10.5))
                .text_color(pal.text_secondary)
                .child(state.t("source_control.clean_tree")),
        )
}
