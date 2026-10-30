//! 3D 模型预览左上角技术统计 HUD 与交互操作快捷键指引

use gpui::*;
use crate::model::model_preview::ModelDocumentState;
use crate::ui::state::AppState;

/// 渲染 3D 画布浮动 HUD (包含格式、平台、顶点数、面数、尺寸与相机距离)
pub fn render_preview_hud(
    state: &AppState,
    doc_state: &ModelDocumentState,
) -> impl IntoElement {
    let pal = state.theme();
    let dims = doc_state.dimensions();
    let platform_label = if doc_state.is_switch { "Switch (NX)" } else { "Wii U (Cafe)" };

    div()
        .absolute()
        .top_3()
        .left_3()
        .p_3()
        .w(px(240.0))
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.panel_border)
        .rounded_lg()
        .shadow_md()
        .flex()
        .flex_col()
        .gap_2()

        // 标题与平台徽章
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(div().text_size(px(14.0)).child("🧊"))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .child(doc_state.model_name.clone()),
                        ),
                )
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_xs()
                        .bg(pal.primary_accent)
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.app_bg)
                        .child(platform_label),
                ),
        )

        // 几何数据统计 (可见数 / 总数)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .p_2()
                .rounded_md()
                .bg(pal.panel_surface)
                .child(render_stat_row(
                    state.t("model_preview.stat_meshes"),
                    format!("{} / {}", doc_state.visible_meshes_count(), doc_state.mesh_items.len()),
                    &pal,
                ))
                .child(render_stat_row(
                    state.t("model_preview.stat_vertices"),
                    format!("{} / {}", doc_state.visible_vertices_count(), doc_state.total_vertices),
                    &pal,
                ))
                .child(render_stat_row(
                    state.t("model_preview.stat_triangles"),
                    format!("{} / {}", doc_state.visible_triangles_count(), doc_state.total_triangles),
                    &pal,
                ))
                .child(render_stat_row(
                    state.t("model_preview.stat_dimensions"),
                    format!("{:.1} × {:.1} × {:.1}", dims[0], dims[1], dims[2]),
                    &pal,
                ))
                .child(render_stat_row(
                    state.t("model_preview.stat_distance"),
                    format!("{:.2} m", doc_state.camera.distance),
                    &pal,
                ))
                .child(render_stat_row(
                    state.t("model_preview.stat_textures"),
                    format!("{}", doc_state.textures.len()),
                    &pal,
                )),
        )

        // 交互操作提示
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .text_size(px(9.5))
                .text_color(pal.text_secondary)
                .child(state.t("model_preview.hint_orbit"))
                .child(state.t("model_preview.hint_pan"))
                .child(state.t("model_preview.hint_zoom")),
        )
}

/// 渲染单行技术统计键值
fn render_stat_row(
    label: &'static str,
    value: String,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(10.5))
        .child(div().text_color(pal.text_secondary).child(label))
        .child(div().font_weight(FontWeight::MEDIUM).text_color(pal.text_primary).child(value))
}
