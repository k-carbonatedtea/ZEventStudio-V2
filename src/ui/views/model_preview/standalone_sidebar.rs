//! 独立模型预览窗口右侧子网格列表与显隐分层侧边栏

use gpui::*;
use crate::ui::theme::ThemePalette;
use super::standalone_window::StandaloneModelPreviewView;

/// 渲染右侧子网格分层侧边栏
pub fn render_standalone_sidebar(
    view_state: &StandaloneModelPreviewView,
    pal: &ThemePalette,
    view: &Entity<StandaloneModelPreviewView>,
) -> impl IntoElement {
    let v_all = view.clone();
    let v_none = view.clone();

    div()
        .w(px(250.0))
        .h_full()
        .bg(pal.panel_surface)
        .border_l_1()
        .border_color(pal.panel_border)
        .flex()
        .flex_col()
        // 顶栏：标题 + 批量显隐开关
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
                        .text_size(px(11.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .child(format!(
                            "🧩 {} ({})",
                            view_state.t("model_preview.meshes_title"),
                            view_state.model_state.mesh_items.len()
                        )),
                )
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
                                .bg(pal.card_surface)
                                .hover(|s| s.bg(pal.card_hover))
                                .text_size(px(10.5))
                                .child("👁️")
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    v_all.update(cx, |this, cx| {
                                        this.set_all_submeshes_visibility(true);
                                        cx.notify();
                                    });
                                }),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_xs()
                                .cursor_pointer()
                                .bg(pal.card_surface)
                                .hover(|s| s.bg(pal.card_hover))
                                .text_size(px(10.5))
                                .child("🚫")
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    v_none.update(cx, |this, cx| {
                                        this.set_all_submeshes_visibility(false);
                                        cx.notify();
                                    });
                                }),
                        ),
                ),
        )
        // 网格列表
        .child(
            div()
                .id("standalone_mesh_list")
                .flex_1()
                .overflow_y_scroll()
                .p_1p5()
                .flex()
                .flex_col()
                .gap_1()
                .children(
                    view_state
                        .model_state
                        .mesh_items
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            let v = view.clone();
                            let is_vis = item.visible;
                            let item_name = item.name.clone();
                            let mat_name = item.material_name.clone();
                            let tris = item.triangle_count;

                            div()
                                .p_1p5()
                                .rounded_md()
                                .cursor_pointer()
                                .bg(if is_vis { pal.card_surface } else { rgba(0x00000010) })
                                .border_1()
                                .border_color(if is_vis { pal.card_border } else { pal.panel_border })
                                .hover(|s| s.bg(pal.card_hover))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    v.update(cx, |this, cx| {
                                        this.toggle_submesh_visibility(idx);
                                        cx.notify();
                                    });
                                })
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .child(if is_vis { "👁️" } else { "🚫" }),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if is_vis { pal.text_primary } else { pal.text_secondary })
                                                .truncate()
                                                .child(item_name),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(9.5))
                                                .text_color(pal.text_secondary)
                                                .truncate()
                                                .child(format!("{} • {} △", mat_name, tris)),
                                        ),
                                )
                        }),
                ),
        )
}
