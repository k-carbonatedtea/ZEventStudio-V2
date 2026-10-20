use gpui::*;
use crate::ui::state::AppState;

/// 渲染顶部工具栏左侧 Branding 区域 (Logo、应用标题)
pub fn render_branding(state: &AppState) -> impl IntoElement {
    let pal = state.theme();

    div()
        .flex()
        .items_center()
        .gap_3()
        // Logo & Title
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_size(px(18.0))
                        .text_color(pal.secondary_accent)
                        .child("⚡"),
                )
                .child(
                    div()
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_size(px(15.0))
                        .text_color(pal.text_primary)
                        .child("NodeFlow Studio"),
                ),
        )
        // Studio Tag
        .child(
            div()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
                .child("EVFL Studio"),
        )
}
