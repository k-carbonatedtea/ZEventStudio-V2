pub mod actions_col;
pub mod details;
pub mod params_col;
pub mod queries_col;
pub mod table;


use gpui::*;
use crate::ui::state::AppState;
use details::render_actor_detail_columns;
use table::render_actors_table;

/// 渲染全屏角色管理工作区视图 (Actor Manager: 角色概览表格 + 下半区动作/查询/参数三栏面板)
pub fn render_actor_manager_view(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let actors = &state.graph.actors;
    let total_count = actors.len();
    let sel_idx = state.selected_actor_idx.min(if total_count > 0 { total_count - 1 } else { 0 });
    let selected_actor = actors.get(sel_idx).cloned();

    let model_add_actor = model.clone();

    let pal = state.theme();

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(pal.app_bg)
        .text_color(pal.text_primary)
        // 顶部工具栏 (角色总数统计 + 添加角色按钮)
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
                        .child(format!("共 {} 个角色", total_count)),
                )
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .hover(move |s| s.bg(pal.card_hover))
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_add_actor.update(cx, |st, cx| {
                                st.is_adding_actor = !st.is_adding_actor;
                                cx.notify();
                            });
                        })
                        .child(state.t("actors_view.add_actor_btn")),
                ),
        )
        // 角色内联添加栏 (若展开)
        .children(if state.is_adding_actor {
            let model_confirm = model.clone();
            let model_cancel = model.clone();
            let model_toggle_dd = model.clone();
            let model_clear = model.clone();

            let actor_presets = [
                "EventSystemActor", "GameROMPlayer", "Npc_Zora001",
                "GameDataCalcMachine", "SceneSoundCtrlTag", "GameRomCamera", "CustomActor",
            ];
            let is_empty = state.new_actor_name.is_empty();
            let show_dd = state.show_actor_dropdown;

            Some(
                div()
                    .px_4()
                    .py_2()
                    .bg(hsla(215.0 / 360.0, 0.35, 0.15, 0.98))
                    .border_b_1()
                    .border_color(hsla(215.0 / 360.0, 0.30, 0.25, 1.0))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_size(px(12.0)).font_weight(FontWeight::BOLD).child("新建角色:"))
                            // 角色名称输入框 + 下拉切换按钮
                            .child(
                                div()
                                    .h(px(28.0))
                                    .w(px(240.0))
                                    .px_2()
                                    .rounded_md()
                                    .bg(hsla(220.0 / 360.0, 0.45, 0.08, 1.0))
                                    .border_1()
                                    .border_color(if show_dd {
                                        hsla(200.0 / 360.0, 0.90, 0.60, 1.0)
                                    } else {
                                        hsla(215.0 / 360.0, 0.80, 0.55, 1.0)
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(if is_empty {
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(crate::ui::components::render_blinking_cursor(hsla(45.0 / 360.0, 0.98, 0.70, 1.0)))
                                            .child(
                                                div()
                                                    .text_size(px(11.5))
                                                    .font_weight(FontWeight::NORMAL)
                                                    .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                                                    .child("输入或选择角色名..."),
                                            )
                                    } else {
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_0p5()
                                            .child(
                                                div()
                                                    .text_size(px(11.5))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(hsla(45.0 / 360.0, 0.98, 0.70, 1.0))
                                                    .child(state.new_actor_name.clone()),
                                            )
                                            .child(crate::ui::components::render_blinking_cursor(hsla(45.0 / 360.0, 0.98, 0.70, 1.0)))
                                    })


                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .children(if !is_empty {
                                                Some(
                                                    div()
                                                        .px_1()
                                                        .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.8)))
                                                        .cursor_pointer()
                                                        .text_size(px(9.0))
                                                        .text_color(hsla(0.0, 0.0, 0.85, 1.0))
                                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                            model_clear.update(cx, |st, cx| {
                                                                st.new_actor_name.clear();
                                                                cx.notify();
                                                            });
                                                        })
                                                        .child("✕"),
                                                )
                                            } else {
                                                None
                                            })
                                            .child(
                                                div()
                                                    .px_1()
                                                    .hover(|s| s.bg(hsla(215.0 / 360.0, 0.35, 0.25, 0.9)))
                                                    .cursor_pointer()
                                                    .text_size(px(9.0))
                                                    .text_color(hsla(215.0 / 360.0, 0.20, 0.85, 1.0))
                                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                        model_toggle_dd.update(cx, |st, cx| {
                                                            st.show_actor_dropdown = !st.show_actor_dropdown;
                                                            cx.notify();
                                                        });
                                                    })
                                                    .child("▼"),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(hsla(145.0 / 360.0, 0.65, 0.35, 1.0))
                                    .hover(|s| s.bg(hsla(145.0 / 360.0, 0.75, 0.42, 1.0)))
                                    .cursor_pointer()
                                    .text_size(px(11.5))
                                    .font_weight(FontWeight::BOLD)
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        model_confirm.update(cx, |st, cx| {
                                            let name = st.new_actor_name.trim().to_string();
                                            if !name.is_empty() {
                                                let mut new_a = crate::model::actor::ActorData::new(name);
                                                new_a.load_default_params();
                                                st.graph.actors.push(new_a);
                                                st.selected_actor_idx = st.graph.actors.len() - 1;
                                            }
                                            st.is_adding_actor = false;
                                            st.show_actor_dropdown = false;
                                            cx.notify();
                                        });
                                    })
                                    .child("✓ 确认添加"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(hsla(0.0, 0.0, 0.25, 1.0))
                                    .hover(|s| s.bg(hsla(0.0, 0.0, 0.35, 1.0)))
                                    .cursor_pointer()
                                    .text_size(px(11.5))
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        model_cancel.update(cx, |st, cx| {
                                            st.is_adding_actor = false;
                                            st.show_actor_dropdown = false;
                                            cx.notify();
                                        });
                                    })
                                    .child("✕ 取消"),
                            ),
                    )
                    // 下拉选项列表
                    .children(if show_dd {
                        Some(
                            div()
                                .id("actor_dropdown_list")
                                .max_h(px(140.0))
                                .overflow_y_scroll()
                                .bg(hsla(220.0 / 360.0, 0.45, 0.10, 0.98))
                                .border_1()
                                .border_color(hsla(215.0 / 360.0, 0.35, 0.30, 1.0))
                                .rounded_md()
                                .shadow_lg()
                                .flex()
                                .flex_col()
                                .children(actor_presets.into_iter().map(|preset| {
                                    let model_item = model.clone();
                                    let p_str = preset.to_string();
                                    div()
                                        .px_2p5()
                                        .py_1p5()
                                        .border_b_1()
                                        .border_color(hsla(220.0 / 360.0, 0.25, 0.16, 0.5))
                                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.50, 0.28, 1.0)))
                                        .cursor_pointer()
                                        .text_size(px(11.0))
                                        .text_color(hsla(215.0 / 360.0, 0.15, 0.90, 1.0))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            let val = p_str.clone();
                                            model_item.update(cx, |st, cx| {
                                                st.new_actor_name = val;
                                                st.show_actor_dropdown = false;
                                                cx.notify();
                                            });
                                        })
                                        .child(preset)
                                })),
                        )
                    } else {
                        None
                    }),
            )
        } else {
            None
        })


        // 上半区：角色概览表格 (Actors Table)
        .child(
            div()
                .id("actors_table_scroll")
                .flex_1()
                .max_h(px(280.0))
                .overflow_y_scroll()
                .bg(pal.app_bg)
                .border_b_1()
                .border_color(pal.panel_border)
                .child(render_actors_table(state, actors, sel_idx, model)),
        )

        // 下半区：所选角色的三栏详情 (动作列表 / 查询列表 / 参数属性)
        .child(
            div()
                .flex_1()
                .size_full()
                .overflow_hidden()
                .flex()
                .bg(pal.app_bg)
                .children(selected_actor.map(|act| {
                    render_actor_detail_columns(state, &act, sel_idx, model)
                })),
        )
}
