use super::card::render_plugin_card;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;
use gpui::*;

/// 渲染左侧导航侧边栏插件中心功能面板
pub fn render_plugins_panel(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let plugins = &state.plugin_manager.plugins;
    let query = state.plugin_search_query.trim().to_lowercase();

    // 过滤插件列表
    let filtered_plugins: Vec<_> = plugins
        .iter()
        .filter(|p| {
            if query.is_empty() {
                return true;
            }
            p.manifest.name.to_lowercase().contains(&query)
                || p.manifest.description.to_lowercase().contains(&query)
                || p.manifest.author.to_lowercase().contains(&query)
                || p.manifest.id.to_lowercase().contains(&query)
        })
        .collect();

    let model_rescan = model.clone();
    let model_open_dir = model.clone();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(pal.panel_surface)
        .overflow_hidden()

        // 1. 顶部操作栏 (标题、插件计数、刷新、打开物理目录)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(pal.panel_border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(12.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .child(state.t("plugins.title")),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .px_1p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(pal.app_bg)
                                .text_color(pal.primary_accent)
                                .child(format!("{}", plugins.len())),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 刷新/重新扫描按钮
                        .child(
                            div()
                                .size(px(26.0))
                                .rounded_md()
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(12.5))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_rescan.update(cx, |st, cx| {
                                        st.rescan_plugins();
                                        cx.notify();
                                    });
                                })
                                .child("🔄"),
                        )
                        // 打开插件物理目录按钮
                        .child(
                            div()
                                .size(px(26.0))
                                .rounded_md()
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(12.5))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_open_dir.update(cx, |st, cx| {
                                        st.open_plugins_folder();
                                        cx.notify();
                                    });
                                })
                                .child("📁"),
                        ),
                ),
        )

        // 2. 搜索过滤输入框 (严格遵循规范 8：具有 on_mouse_down_out 自动失焦与清空按钮)
        .child(render_plugin_search_input(state, model))

        // 3. 插件列表主体可滚动区域
        .child(
            div()
                .id("plugins_scroll_body")
                .flex_1()
                .overflow_y_scroll()
                .p_2p5()
                .flex()
                .flex_col()
                .gap_2p5()
                .children(if filtered_plugins.is_empty() {
                    vec![render_empty_plugins_view(state).into_any_element()]
                } else {
                    filtered_plugins
                        .into_iter()
                        .map(|plugin| render_plugin_card(plugin, state, model).into_any_element())
                        .collect()
                }),
        )
}

/// 渲染插件搜索输入框 (遵循三步输入规范与 .on_mouse_down_out 强制失焦)
fn render_plugin_search_input(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_focused = state.plugin_search_focused;
    let query = state.plugin_search_query.clone();

    let model_click = model.clone();
    let model_out = model.clone();
    let model_clear = model.clone();

    div()
        .px_2p5()
        .pt_2()
        .child(
            div()
                .id("plugin_search_input_container")
                .cursor_text()
                .flex()
                .items_center()
                .justify_between()
                .px_2p5()
                .py_1()
                .rounded_lg()
                .bg(pal.app_bg)
                .border_1()
                .border_color(if is_focused {
                    pal.primary_accent
                } else {
                    pal.panel_border
                })
                // 1. 点击获得焦点并清空其他输入框焦点
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_click.update(cx, |st, cx| {
                        st.clear_active_text_inputs();
                        st.plugin_search_focused = true;
                        cx.notify();
                    });
                })
                // 2. 点击外部区域自动失焦
                .on_mouse_down_out(move |_, _, cx| {
                    model_out.update(cx, |st, cx| {
                        if st.plugin_search_focused {
                            st.plugin_search_focused = false;
                            cx.notify();
                        }
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .flex_1()
                        .child(div().text_size(px(11.0)).child("🔍"))
                        .child(
                            if query.is_empty() && !is_focused {
                                div()
                                    .text_size(px(11.0))
                                    .text_color(pal.text_secondary)
                                    .child(state.t("plugins.search_placeholder"))
                                    .into_any_element()
                            } else {
                                div()
                                    .flex()
                                    .items_center()
                                    .text_size(px(11.0))
                                    .text_color(pal.text_primary)
                                    .child(query.clone())
                                    .children(is_focused.then(|| {
                                        render_blinking_cursor(pal.primary_accent.into())
                                            .into_any_element()
                                    }))
                                    .into_any_element()
                            },
                        ),
                )
                // 3. 非空时渲染一键清空按钮
                .children((!query.is_empty()).then(|| {
                    div()
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.text_color(pal.text_primary))
                        .child("✕")
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_clear.update(cx, |st, cx| {
                                st.plugin_search_query.clear();
                                cx.notify();
                            });
                        })
                })),
        )
}

/// 渲染未找到或无可用插件时的空态提示
fn render_empty_plugins_view(state: &AppState) -> impl IntoElement {
    let pal = state.theme();
    div()
        .w_full()
        .py_10()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(div().text_size(px(32.0)).child("🧩"))
        .child(
            div()
                .text_size(px(12.5))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
                .child(state.t("plugins.no_plugins")),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(pal.text_secondary)
                .child(state.t("plugins.empty_hint")),
        )
}
