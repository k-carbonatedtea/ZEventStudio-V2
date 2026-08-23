use gpui::*;
use crate::model::PropertyItem;
use crate::ui::state::AppState;

/// 渲染选中节点的 EVFL 参数属性列表卡片
pub fn render_prop_list(
    state: &AppState,
    properties: &[PropertyItem],
) -> impl IntoElement {
    let pal = state.theme();
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        // 标题与参数计数
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_primary) // 亮白灰 Text Primary
                .child(format!("{} ({})", state.t("inspector.parameters"), properties.len())),
        )
        // 属性条目列表
        .children(if properties.is_empty() {
            vec![div()
                .p_2()
                .rounded_md()
                .bg(pal.card_surface)
                .text_size(px(10.5))
                .text_color(pal.text_secondary)
                .child(state.t("inspector.no_parameters"))]
        } else {
            properties
                .iter()
                .map(|prop| {
                    let (badge_bg, badge_text) = match prop.val_type.as_str() {
                        "string" => (rgb(0x10B981), "STR"),
                        "number" | "int" => (rgb(0x00C4FF), "INT"),
                        "float" => (rgb(0x8B5CF6), "FLT"),
                        "bool" => (rgb(0xF59E0B), "BOOL"),
                        _ => (rgb(0x64748B), "VAL"),
                    };

                    div()
                        .p_2()
                        .rounded_md()
                        .bg(pal.card_surface) // 浅层蓝灰 Card / Hover
                        .border_1()
                        .border_color(pal.card_border)
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_primary) // 亮白灰 Text Primary
                                        .child(prop.key.clone()),
                                )
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(badge_bg)
                                        .text_size(px(8.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xFFFFFF))
                                        .child(badge_text),
                                    ),
                            )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(pal.secondary_accent) // 琥珀金 Secondary Accent
                                .child(prop.val.clone()),
                        )
                })
                .collect()
        })
}
