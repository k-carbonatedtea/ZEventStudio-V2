use gpui::*;
use crate::ui::state::AppState;

/// 渲染 VS Code 风格的源文件管理侧边栏面板 (Source Control Panel)
pub fn render_source_control_panel(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let modified_tabs: Vec<_> = state
        .open_tabs
        .iter()
        .filter(|t| t.is_modified)
        .cloned()
        .collect();

    let dirty_count = modified_tabs.len();
    let model_save_all = model.clone();
    let model_discard_all = model.clone();
    let model_refresh = model.clone();

    div()
        .flex()
        .flex_col()
        .h_full()
        .w_full()
        .bg(pal.panel_surface)
        .overflow_hidden()

        // 1. 顶部源文件管理标题栏与全局操作按钮组
        .child(
            div()
                .h(px(36.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(pal.panel_border)
                .bg(pal.panel_surface)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.0))
                                .text_color(pal.text_primary)
                                .child(state.t("source_control.title")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 批量保存全部修改按钮 (💾)
                        .children((dirty_count > 0).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.primary_accent)
                                .hover(move |s| s.bg(pal.card_hover))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_save_all.update(cx, |st, cx| {
                                        st.save_all_modified_documents();
                                        cx.notify();
                                    });
                                })
                                .child("💾")
                        }))
                        // 批量放弃全部修改按钮 (↩)
                        .children((dirty_count > 0).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.secondary_accent)
                                .hover(move |s| s.bg(pal.card_hover))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_discard_all.update(cx, |st, cx| {
                                        st.discard_all_modified_documents();
                                        cx.notify();
                                    });
                                })
                                .child("↩")
                        }))
                        // 刷新状态按钮 (🔄)
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(move |s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_refresh.update(cx, |st, cx| {
                                        st.sync_active_tab_modified();
                                        cx.notify();
                                    });
                                })
                                .child("🔄"),
                        ),
                ),
        )

        // 2. 更改列表分节标题 (CHANGES)
        .child(
            div()
                .px_3()
                .py_1p5()
                .bg(pal.card_surface)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_secondary)
                                .child(state.t("source_control.changes")),
                        )
                        .children((dirty_count > 0).then(|| {
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
                                .child(dirty_count.to_string())
                        })),
                ),
        )

        // 3. 修改文件列表项 / 空状态展示
        .child(
            div()
                .id("source_control_scroll")
                .flex_1()
                .overflow_y_scroll()
                .child(if dirty_count == 0 {
                    // 无未保存更改空状态引导
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
                                .child(state.t("source_control.no_changes")),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("source_control.all_saved_hint")),
                        )
                        .into_any_element()
                } else {
                    // 已修改文档条目列表
                    div()
                        .p_1()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .children(modified_tabs.into_iter().map(|tab| {
                            let tab_id = tab.id.clone();
                            let model_row = model.clone();
                            let model_save = model.clone();
                            let model_discard = model.clone();
                            let tid_save = tab.id.clone();
                            let tid_discard = tab.id.clone();

                            let dir_path = std::path::Path::new(&tab.file_path)
                                .parent()
                                .and_then(|p| p.to_str())
                                .unwrap_or("")
                                .to_string();

                            div()
                                .id(ElementId::Name(format!("sc_tab_{}", tab_id).into()))
                                .group("sc-row")
                                .px_2()
                                .py_1p5()
                                .rounded_md()
                                .cursor_pointer()
                                .hover(|s| s.bg(pal.card_hover))
                                .flex()
                                .items_center()
                                .justify_between()
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let id = tab_id.clone();
                                    model_row.update(cx, |st, cx| {
                                        st.switch_document_tab(&id);
                                        cx.notify();
                                    });
                                })
                                // 左侧图标与文件名
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .flex_shrink_0()
                                                .child(tab.icon()),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .overflow_hidden()
                                                .child(
                                                    div()
                                                        .text_size(px(11.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(pal.secondary_accent) // 琥珀金高亮
                                                        .overflow_hidden()
                                                        .whitespace_nowrap()
                                                        .child(tab.title),
                                                )
                                                .children((!dir_path.is_empty()).then(|| {
                                                    div()
                                                        .text_size(px(9.5))
                                                        .text_color(pal.text_secondary)
                                                        .overflow_hidden()
                                                        .whitespace_nowrap()
                                                        .child(dir_path)
                                                })),
                                        ),
                                )
                                // 右侧操作按钮组与 'M' 状态徽标
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .flex_shrink_0()
                                        // 放弃修改快捷按钮 (悬浮可见)
                                        .child(
                                            div()
                                                .px_1()
                                                .py_0p5()
                                                .rounded_xs()
                                                .cursor_pointer()
                                                .text_size(px(10.0))
                                                .text_color(pal.secondary_accent)
                                                .hover(move |s| s.bg(pal.card_surface))
                                                .on_mouse_down(MouseButton::Left, move |_e, _, cx| {
                                                    cx.stop_propagation();
                                                    let id = tid_discard.clone();
                                                    model_discard.update(cx, |st, cx| {
                                                        st.discard_document_changes(&id);
                                                        cx.notify();
                                                    });
                                                })
                                                .child("↩")
                                        )
                                        // 单个保存快捷按钮 (悬浮可见)
                                        .child(
                                            div()
                                                .px_1()
                                                .py_0p5()
                                                .rounded_xs()
                                                .cursor_pointer()
                                                .text_size(px(10.0))
                                                .text_color(pal.primary_accent)
                                                .hover(move |s| s.bg(pal.card_surface))
                                                .on_mouse_down(MouseButton::Left, move |_e, _, cx| {
                                                    cx.stop_propagation();
                                                    let id = tid_save.clone();
                                                    model_save.update(cx, |st, cx| {
                                                        st.save_document_tab(&id);
                                                        cx.notify();
                                                    });
                                                })
                                                .child("💾")
                                        )
                                        // VS Code 经典 Modified 'M' 状态标记
                                        .child(
                                            div()
                                                .w(px(16.0))
                                                .h(px(16.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(pal.secondary_accent)
                                                .child("M")
                                        )
                                )
                        }))
                        .into_any_element()
                }),
        )
}
