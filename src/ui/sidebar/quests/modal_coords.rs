use gpui::*;
use crate::ui::components::Toast;
use crate::ui::sidebar::quests::input_helper::render_modal_text_field;
use crate::ui::state::{AppState, QuestCreatorFocusField, QuestCreatorModalState};

/// 渲染任务向导：地图坐标与标点指示器选项卡
pub fn render_tab_coords(
    modal: &QuestCreatorModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_toggle = model.clone();
    let model_pick = model.clone();
    let user_pin = state.quest_map_user_pin;

    div()
        .flex()
        .flex_col()
        .gap_3()
        // 1. 标点启用开关与地图快速拾取坐标条
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p_2p5()
                .rounded_sm()
                .bg(pal.card_hover.opacity(0.6))
                .border_1()
                .border_color(pal.panel_border.opacity(0.6))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_toggle.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.quest_creator_modal {
                                    m.enable_indicator = !m.enable_indicator;
                                    cx.notify();
                                }
                            });
                        })
                        .child(
                            div()
                                .size(px(16.0))
                                .rounded_sm()
                                .border_1()
                                .border_color(if modal.enable_indicator { pal.primary_accent } else { pal.panel_border })
                                .bg(if modal.enable_indicator { pal.primary_accent } else { pal.panel_surface })
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(11.0))
                                .text_color(rgb(0x000000))
                                .child(if modal.enable_indicator { "✓" } else { "" }),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(if modal.enable_indicator { pal.text_primary } else { pal.text_secondary })
                                .child(state.t("quests.enable_indicator")),
                        ),
                )
                // 从当前 2D 地图标点拾取坐标按钮
                .child(
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded_sm()
                        .border_1()
                        .border_color(if user_pin.is_some() { pal.primary_accent } else { pal.panel_border })
                        .bg(if user_pin.is_some() { pal.primary_accent.opacity(0.15) } else { pal.panel_surface })
                        .text_color(if user_pin.is_some() { pal.primary_accent } else { pal.text_secondary })
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_pick.update(cx, |st, cx| {
                                if st.fill_quest_creator_from_map_pin() {
                                    st.toast = Some(Toast::success(st.t("quests.coords_picked_toast")));
                                } else {
                                    st.toast = Some(Toast::warning(st.t("quests.no_map_pin_toast")));
                                }
                                cx.notify();
                            });
                        })
                        .child(if let Some((wx, wz)) = user_pin {
                            format!("📍 拾取当前标点 ({:.0}, {:.0})", wx, wz)
                        } else {
                            state.t("quests.btn_pick_map_coord").to_string()
                        }),
                ),
        )
        // 2. 目标 Actor
        .child(
            div()
                .flex()
                .gap_3()
                .child(render_modal_text_field(
                    state.t("quests.field_indicator_actor"),
                    &modal.indicator_actor,
                    QuestCreatorFocusField::IndicatorActor,
                    if modal.orderer.is_empty() { "Npc_AncientDoctor_Robbie" } else { &modal.orderer },
                    state,
                    model,
                ))
                .child(render_modal_text_field(
                    state.t("quests.field_off_flag"),
                    &modal.indicator_off_flag,
                    QuestCreatorFocusField::IndicatorOffFlag,
                    "Quest_Off_Flag (可选)",
                    state,
                    model,
                )),
        )
        // 3. 世界坐标三元组 (X, Y, Z)
        .child(
            div()
                .flex()
                .gap_2()
                .child(render_modal_text_field(
                    state.t("quests.field_coord_x"),
                    &modal.coord_x,
                    QuestCreatorFocusField::CoordX,
                    "0.0",
                    state,
                    model,
                ))
                .child(render_modal_text_field(
                    state.t("quests.field_coord_y"),
                    &modal.coord_y,
                    QuestCreatorFocusField::CoordY,
                    "0.0",
                    state,
                    model,
                ))
                .child(render_modal_text_field(
                    state.t("quests.field_coord_z"),
                    &modal.coord_z,
                    QuestCreatorFocusField::CoordZ,
                    "0.0",
                    state,
                    model,
                )),
        )
        // 4. 底部说明提示
        .child(
            div()
                .text_size(px(11.0))
                .text_color(pal.text_secondary.opacity(0.8))
                .child(format!("💡 {}", state.t("quests.btn_pick_map_coord_tip"))),
        )
}
