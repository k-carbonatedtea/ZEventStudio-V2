use gpui::*;
use crate::ui::state::AppState;

/// 渲染全屏 JSON 源码与 EVFL Schema 视图
pub fn render_json_viewer_view(
    state: &AppState,
    _model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let json_text = serde_json::to_string_pretty(&state.graph).unwrap_or_else(|_| "{}".to_string());
    let line_count = json_text.lines().count();

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(pal.app_bg)
        .text_color(pal.text_primary)
        // 顶部信息栏
        .child(
            div()
                .h(px(40.0))
                .px_4()
                .border_b_1()
                .border_color(pal.panel_border)
                .flex()
                .items_center()
                .justify_between()
                .bg(pal.panel_surface)
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .child(format!("JSON 源码定义 ({} 行, {} 字节)", line_count, json_text.len())),
                ),
        )
        // 代码文本区域
        .child(
            div()
                .id("json_viewer_scroll")
                .flex_1()
                .size_full()
                .p_4()
                .overflow_y_scroll()
                .bg(pal.app_bg)
                .font_family(".SystemUIFont")
                .text_size(px(12.0))
                .text_color(pal.text_primary)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .children(json_text.lines().take(500).enumerate().map(|(l_idx, line)| {
                            div()
                                .flex()
                                .gap_3()
                                .child(
                                    div()
                                        .w(px(40.0))
                                        .text_color(pal.text_secondary)
                                        .child(format!("{}", l_idx + 1)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .text_color(if line.trim().starts_with('"') && line.contains(':') {
                                            pal.secondary_accent
                                        } else {
                                            pal.text_primary
                                        })
                                        .child(line.to_string()),
                                )
                        })),
                ),
        )
}
