//! 3D 模型预览右侧子网格列表与显隐分层侧边栏

use gpui::*;
use crate::model::model_preview::ModelDocumentState;
use crate::ui::state::AppState;

/// 渲染 3D 模型子网格 (Submesh) 列表侧边栏
pub fn render_mesh_sidebar(
    state: &AppState,
    doc_state: &ModelDocumentState,
    tab_id: &str,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let tid = tab_id.to_string();
    let selected_idx = doc_state.selected_mesh_idx;
    let query = doc_state.filter_query.to_lowercase();

    div()
        .w(px(250.0))
        .h_full()
        .bg(pal.panel_surface)
        .border_l_1()
        .border_color(pal.panel_border)
        .flex()
        .flex_col()

        // 顶栏：标题 + 批量显隐控制
        .child(
            div()
                .h(px(38.0))
                .px_3()
                .border_b_1()
                .border_color(pal.panel_border)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .text_size(px(11.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .child("🧩")
                        .child(format!("{} ({})", state.t("model_preview.meshes_title"), doc_state.mesh_items.len())),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(render_batch_vis_btn("👁️", &tid, model, true, &pal))
                        .child(render_batch_vis_btn("🚫", &tid, model, false, &pal)),
                ),
        )

        // 搜索过滤栏
        .child(
            div()
                .p_2()
                .border_b_1()
                .border_color(pal.panel_border)
                .child(
                    div()
                        .h(px(24.0))
                        .px_2()
                        .rounded_xs()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(if doc_state.search_focused { pal.primary_accent } else { pal.panel_border })
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(div().text_size(px(10.0)).text_color(pal.text_secondary).child("🔍"))
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(10.5))
                                .text_color(if doc_state.filter_query.is_empty() { pal.text_secondary } else { pal.text_primary })
                                .child(if doc_state.filter_query.is_empty() {
                                    state.t("model_preview.filter_placeholder").to_string()
                                } else {
                                    doc_state.filter_query.clone()
                                }),
                        ),
                ),
        )

        // 子网格条目滚动列表
        .child(
            div()
                .id("mesh_sidebar_list")
                .flex_1()
                .overflow_y_scroll()
                .p_1p5()
                .flex()
                .flex_col()
                .gap_1()
                .children(
                    doc_state.mesh_items.iter().enumerate().filter(|(_, item)| {
                        if query.is_empty() {
                            true
                        } else {
                            item.name.to_lowercase().contains(&query)
                                || item.material_name.to_lowercase().contains(&query)
                        }
                    }).map(|(idx, item)| {
                        let is_selected = selected_idx == Some(idx);
                        let m_vis = model.clone();
                        let m_sel = model.clone();
                        let t_vis = tid.clone();
                        let t_sel = tid.clone();

                        div()
                            .h(px(32.0))
                            .px_2()
                            .rounded_xs()
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_between()
                            .bg(if is_selected { pal.card_hover } else { rgba(0x00000000) })
                            .border_1()
                            .border_color(if is_selected { pal.primary_accent } else { rgba(0x00000000) })
                            .hover(|s| s.bg(pal.card_surface))
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                let t = t_sel.clone();
                                m_sel.update(cx, |st, cx| {
                                    if let Some(tab) = st.open_tabs.iter_mut().find(|x| x.id == t) {
                                        if let Some(ref mut ms) = tab.model_state {
                                            ms.selected_mesh_idx = if ms.selected_mesh_idx == Some(idx) {
                                                None
                                            } else {
                                                Some(idx)
                                            };
                                            cx.notify();
                                        }
                                    }
                                });
                            })
                            // 左侧：可见性眼球 + 网格名与材质名
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .cursor_pointer()
                                            .text_size(px(11.0))
                                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                cx.stop_propagation();
                                                let t = t_vis.clone();
                                                m_vis.update(cx, |st, cx| {
                                                    if let Some(tab) = st.open_tabs.iter_mut().find(|x| x.id == t) {
                                                        if let Some(ref mut ms) = tab.model_state {
                                                            ms.toggle_mesh_visibility(idx);
                                                            cx.notify();
                                                        }
                                                    }
                                                });
                                            })
                                            .child(if item.visible { "👁️" } else { "🔒" }),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .overflow_hidden()
                                            .child(
                                                div()
                                                    .text_size(px(10.5))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(if item.visible { pal.text_primary } else { pal.text_secondary })
                                                    .whitespace_nowrap()
                                                    .child(item.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(9.0))
                                                    .text_color(pal.text_secondary)
                                                    .whitespace_nowrap()
                                                    .child(if let Some(ref tex) = item.texture_name {
                                                        format!("{} · 🖼️ {}", item.material_name, tex)
                                                    } else {
                                                        item.material_name.clone()
                                                    }),
                                            ),
                                    ),
                            )
                            // 右侧：三角面数微型徽章
                            .child(
                                div()
                                    .px_1()
                                    .py_0p5()
                                    .rounded_xs()
                                    .bg(pal.card_surface)
                                    .text_size(px(8.5))
                                    .text_color(pal.text_secondary)
                                    .child(format!("{} △", item.triangle_count)),
                            )
                    })
                ),
        )
}

/// 渲染全部显示/隐藏小按钮
fn render_batch_vis_btn(
    icon: &'static str,
    tab_id: &str,
    model: &Entity<AppState>,
    visible: bool,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let m = model.clone();
    let tid = tab_id.to_string();

    div()
        .w(px(22.0))
        .h(px(22.0))
        .rounded_xs()
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(10.0))
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.panel_border)
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let t = tid.clone();
            m.update(cx, |st, cx| {
                if let Some(tab) = st.open_tabs.iter_mut().find(|x| x.id == t) {
                    if let Some(ref mut ms) = tab.model_state {
                        ms.set_all_visibility(visible);
                        cx.notify();
                    }
                }
            });
        })
        .child(icon)
}
