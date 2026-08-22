use gpui::*;
use crate::ui::state::AppState;

/// 渲染全屏 JSON 源码与 EVFL Schema 视图
pub fn render_json_viewer_view(
    state: &AppState,
    _model: &Entity<AppState>,
) -> impl IntoElement {
    let json_text = serde_json::to_string_pretty(&state.graph).unwrap_or_else(|_| "{}".to_string());
    let line_count = json_text.lines().count();

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(hsla(220.0 / 360.0, 0.35, 0.08, 1.0))
        .text_color(rgb(0xffffff))
        // 顶部信息栏
        .child(
            div()
                .h(px(40.0))
                .px_4()
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
                .flex()
                .items_center()
                .justify_between()
                .bg(hsla(220.0 / 360.0, 0.30, 0.10, 0.95))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(hsla(0.0, 0.0, 1.0, 0.95))
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
                .bg(hsla(220.0 / 360.0, 0.35, 0.06, 1.0))

                .font_family(".SystemUIFont")
                .text_size(px(12.0))
                .text_color(hsla(200.0 / 360.0, 0.85, 0.80, 1.0))
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
                                        .text_color(hsla(215.0 / 360.0, 0.20, 0.45, 1.0))
                                        .child(format!("{}", l_idx + 1)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .text_color(if line.trim().starts_with('"') && line.contains(':') {
                                            hsla(45.0 / 360.0, 0.90, 0.75, 1.0)
                                        } else {
                                            hsla(215.0 / 360.0, 0.20, 0.85, 1.0)
                                        })
                                        .child(line.to_string()),
                                )
                        })),
                ),
        )
}
