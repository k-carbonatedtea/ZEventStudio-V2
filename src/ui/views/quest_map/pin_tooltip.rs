use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::quest_map::pins_layer::RenderedPinItem;
use crate::ui::views::quest_map::types::{get_region_name_for_coord, get_section_for_coord};

/// 渲染当前高亮/选中标点的玻璃拟态详情悬浮浮窗
pub fn render_pin_tooltip_layer(
    pins: &[RenderedPinItem],
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let is_zh = state.lang == crate::i18n::Language::Zh;

    // 优先显示选中的标点，其次显示鼠标悬停的标点
    let active_pin = state.quest_map_selected_pin.as_ref()
        .or(state.quest_map_hovered_pin.as_ref())
        .and_then(|(qn, s, i)| {
            pins.iter().find(|p| &p.quest_name == qn && p.step_index == *s && p.indicator_index == *i)
        });

    div()
        .absolute()
        .inset_0()
        .children(active_pin.map(|pin| {
            let (sx, sy) = pin.screen_pos;
            let ind = &pin.indicator;
            let coords_str = format!("X: {:.0}, Y: {:.0}, Z: {:.0}", ind.position.0, ind.position.1, ind.position.2);
            let section_str = get_section_for_coord(ind.position.0, ind.position.2);
            let region_str = get_region_name_for_coord(ind.position.0, ind.position.2, is_zh);

            let model_focus = model.clone();
            let model_copy = model.clone();
            let model_close = model.clone();
            let target_pos = ind.position;

            let (badge_bg, badge_fg) = match pin.quest_type {
                crate::model::quest::QuestType::Main => (pal.secondary_accent.opacity(0.2), pal.secondary_accent),
                crate::model::quest::QuestType::Sub => (pal.primary_accent.opacity(0.2), pal.primary_accent),
                crate::model::quest::QuestType::Mini => (pal.primary_accent_hover.opacity(0.2), pal.primary_accent_hover),
            };

            div()
                .id("pin_tooltip_card")
                .absolute()
                .left(px((sx + 18.0).clamp(10.0, 1200.0)))
                .top(px((sy - 90.0).clamp(10.0, 800.0)))
                .w(px(260.0))
                .p_2p5()
                .rounded_lg()
                .bg(pal.panel_surface.opacity(0.96))
                .border_1()
                .border_color(pal.primary_accent.opacity(0.6))
                .shadow_lg()
                .flex()
                .flex_col()
                .gap_1p5()
                // 1. 标题栏与关闭
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(badge_bg)
                                        .text_color(badge_fg)
                                        .child(state.t(pin.quest_type.label_key())),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_primary)
                                        .child(pin.quest_title.clone()),
                                ),
                        )
                        .child(
                            div()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(pal.text_primary))
                                .child("✕")
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_close.update(cx, |st, cx| {
                                        st.quest_map_selected_pin = None;
                                        cx.notify();
                                    });
                                }),
                        ),
                )
                // 2. 阶段名与目标 Actor
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .child(format!("{} #{}: {}", state.t("quests.steps_header"), pin.step_index + 1, pin.step_name))
                        .children(ind.actor_name.as_ref().map(|act| {
                            div()
                                .text_color(pal.primary_accent)
                                .font_weight(FontWeight::MEDIUM)
                                .child(act.clone())
                        })),
                )
                // 3. 详细坐标与分区
                .child(
                    div()
                        .p_1p5()
                        .rounded_md()
                        .bg(pal.card_hover.opacity(0.6))
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .text_size(px(10.5))
                                .child(div().text_color(pal.text_secondary).child(state.t("quests.coords_label")))
                                .child(div().text_color(pal.text_primary).font_weight(FontWeight::SEMIBOLD).child(coords_str.clone())),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .text_size(px(10.5))
                                .child(div().text_color(pal.text_secondary).child(state.t("quests.map_section")))
                                .child(div().text_color(pal.secondary_accent).child(format!("{} • {}", section_str, region_str))),
                        )
                        .children(ind.off_flag.as_ref().map(|flag| {
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .text_size(px(10.0))
                                .child(div().text_color(pal.text_secondary).child(state.t("quests.off_flag_label")))
                                .child(div().text_color(pal.text_secondary).child(flag.clone()))
                        })),
                )
                // 4. 底部快捷操作 (聚焦 & 复制坐标)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_1p5()
                        .pt_1()
                        .child(
                            div()
                                .cursor_pointer()
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .bg(pal.panel_border.opacity(0.4))
                                .text_size(px(10.5))
                                .text_color(pal.text_primary)
                                .hover(|s| s.bg(pal.card_hover))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let c_str = coords_str.clone();
                                    model_copy.update(cx, |st, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(c_str));
                                        st.toast = Some(crate::ui::components::Toast::info(st.t("quests.map_coords_copied")));
                                        cx.notify();
                                    });
                                })
                                .child(state.t("quests.map_copy_coords")),
                        )
                        .child(
                            div()
                                .cursor_pointer()
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .bg(pal.primary_accent.opacity(0.2))
                                .text_size(px(10.5))
                                .text_color(pal.primary_accent)
                                .hover(|s| s.bg(pal.primary_accent.opacity(0.35)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_focus.update(cx, |st, cx| {
                                        st.focus_pin_on_map(target_pos);
                                        cx.notify();
                                    });
                                })
                                .child(state.t("quests.map_focus_pin")),
                        ),
                )
        }))
}
