use gpui::*;
use crate::model::PropertyItem;
use crate::ui::state::AppState;

/// 渲染选中节点的 EVFL 参数属性列表卡片
pub fn render_prop_list(
    state: &AppState,
    properties: &[PropertyItem],
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        // 标题与参数计数
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(hsla(215.0 / 360.0, 0.2, 0.7, 1.0))
                .child(format!("{} ({})", state.t("inspector.parameters"), properties.len())),
        )
        // 属性条目列表
        .children(if properties.is_empty() {
            vec![div()
                .p_2()
                .rounded_md()
                .bg(hsla(220.0 / 360.0, 0.2, 0.15, 1.0))
                .text_size(px(10.5))
                .text_color(hsla(215.0 / 360.0, 0.2, 0.5, 1.0))
                .child(state.t("inspector.no_parameters"))]
        } else {
            properties
                .iter()
                .map(|prop| {
                    let (badge_bg, badge_text) = match prop.val_type.as_str() {
                        "string" => (hsla(145.0 / 360.0, 0.5, 0.25, 1.0), "STR"),
                        "number" | "int" => (hsla(200.0 / 360.0, 0.5, 0.25, 1.0), "INT"),
                        "float" => (hsla(280.0 / 360.0, 0.5, 0.25, 1.0), "FLT"),
                        "bool" => (hsla(40.0 / 360.0, 0.5, 0.25, 1.0), "BOOL"),
                        _ => (hsla(215.0 / 360.0, 0.3, 0.25, 1.0), "VAL"),
                    };

                    div()
                        .p_2()
                        .rounded_md()
                        .bg(hsla(220.0 / 360.0, 0.25, 0.15, 1.0))
                        .border_1()
                        .border_color(hsla(215.0 / 360.0, 0.25, 0.22, 1.0))
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
                                        .text_color(hsla(215.0 / 360.0, 0.3, 0.75, 1.0))
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
                                        .text_color(rgb(0xffffff))
                                        .child(badge_text),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(hsla(45.0 / 360.0, 0.85, 0.80, 1.0))
                                .child(prop.val.clone()),
                        )
                })
                .collect()
        })
}
