use gpui::*;
use crate::ui::state::AppState;

/// 渲染参数属性列 (Parameters Column: 列表展示 + 默认参数注入 + 下拉选择操作条)
pub fn render_params_column(
    state: &AppState,
    actor: &crate::model::actor::ActorData,
    actor_idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div()
        .flex_1()
        .h_full()
        .flex()
        .flex_col()
        .child(render_params_header(state, actor_idx, model))
        .children(if state.is_adding_param {
            Some(render_inline_param_bar(state, actor_idx, model))
        } else {
            None
        })
        .child(
            div()
                .id("actor_params_scroll")
                .flex_1()
                .overflow_y_scroll()
                .child(render_params_table(state, &actor.params, actor_idx, model)),
        )
}

fn render_params_header(
    state: &AppState,
    actor_idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_load = model.clone();
    let model_add = model.clone();
    let is_adding = state.is_adding_param;

    div()
        .h(px(34.0))
        .px_4()
        .bg(hsla(220.0 / 360.0, 0.30, 0.12, 1.0))
        .border_b_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.18, 1.0))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(12.0))
                .text_color(hsla(0.0, 0.0, 1.0, 0.95))
                .child(state.t("actors_view.params_title")),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(hsla(200.0 / 360.0, 0.40, 0.25, 1.0))
                        .hover(|s| s.bg(hsla(200.0 / 360.0, 0.55, 0.35, 1.0)))
                        .cursor_pointer()
                        .text_size(px(10.5))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_load.update(cx, |st, cx| {
                                if let Some(act) = st.graph.actors.get_mut(actor_idx) {
                                    act.load_default_params();
                                }
                                cx.notify();
                            });
                        })
                        .child(state.t("actors_view.load_default_params")),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(if is_adding {
                            hsla(0.0, 0.0, 0.25, 1.0)
                        } else {
                            hsla(215.0 / 360.0, 0.35, 0.22, 1.0)
                        })
                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.50, 0.32, 1.0)))
                        .cursor_pointer()
                        .text_size(px(10.5))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_add.update(cx, |st, cx| {
                                st.is_adding_param = !st.is_adding_param;
                                st.new_param_key = String::new();
                                st.show_param_dropdown = false;
                                cx.notify();
                            });
                        })
                        .child(if is_adding { "✕ 取消" } else { "+ 添加..." }),
                ),
        )
}

fn render_inline_param_bar(
    state: &AppState,
    actor_idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let key_suggestions = ["CreateMode", "IsWorld", "PosX", "PosY", "PosZ", "FlagName", "MessageId", "IsGrounding"];
    let types = ["Int", "Bool", "Float", "String"];
    let model_confirm = model.clone();
    let model_toggle_dd = model.clone();
    let model_clear = model.clone();

    let current_key = &state.new_param_key;
    let current_type = &state.new_param_type;
    let current_val = &state.new_param_val;
    let is_empty = current_key.is_empty();
    let show_dd = state.show_param_dropdown;

    div()
        .p_2()
        .bg(hsla(215.0 / 360.0, 0.35, 0.16, 0.98))
        .border_b_1()
        .border_color(hsla(215.0 / 360.0, 0.30, 0.25, 1.0))
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                // 类型切换胶囊组
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .children(types.into_iter().map(|t_item| {
                            let model_t = model.clone();
                            let is_sel = current_type == t_item;
                            let t_str = t_item.to_string();
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(if is_sel { hsla(200.0 / 360.0, 0.60, 0.35, 1.0) } else { hsla(220.0 / 360.0, 0.25, 0.15, 1.0) })
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let v = t_str.clone();
                                    model_t.update(cx, |st, cx| {
                                        st.new_param_type = v;
                                        cx.notify();
                                    });
                                })
                                .child(t_item)
                        })),
                )
                // 键名输入框 + 下拉切换按钮
                .child(
                    div()
                        .flex_1()
                        .h(px(26.0))
                        .px_2()
                        .rounded_md()
                        .bg(hsla(220.0 / 360.0, 0.45, 0.08, 1.0))
                        .border_1()
                        .border_color(hsla(215.0 / 360.0, 0.80, 0.55, 1.0))
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
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                                        .child("输入或选择键名..."),
                                )
                        } else {
                            div()
                                .flex()
                                .items_center()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(hsla(45.0 / 360.0, 0.98, 0.70, 1.0))
                                        .child(current_key.clone()),
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
                                            .text_size(px(8.5))
                                            .text_color(hsla(0.0, 0.0, 0.85, 1.0))
                                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                model_clear.update(cx, |st, cx| {
                                                    st.new_param_key.clear();
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
                                                st.show_param_dropdown = !st.show_param_dropdown;
                                                cx.notify();
                                            });
                                        })
                                        .child("▼"),
                                ),
                        ),
                )
                // 默认初始值
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(hsla(220.0 / 360.0, 0.40, 0.10, 1.0))
                        .text_size(px(11.0))
                        .text_color(hsla(45.0 / 360.0, 0.90, 0.70, 1.0))
                        .child(current_val.to_string()),
                )
                // 确认添加按钮
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(hsla(145.0 / 360.0, 0.65, 0.35, 1.0))
                        .hover(|s| s.bg(hsla(145.0 / 360.0, 0.75, 0.42, 1.0)))
                        .cursor_pointer()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_confirm.update(cx, |st, cx| {
                                let key_name = st.new_param_key.trim().to_string();
                                if !key_name.is_empty() {
                                    if let Some(act) = st.graph.actors.get_mut(actor_idx) {
                                        act.params.push(crate::model::node::PropertyItem {
                                            key: key_name,
                                            val: if st.new_param_val.trim().is_empty() { "0".to_string() } else { st.new_param_val.trim().to_string() },
                                            val_type: st.new_param_type.clone(),
                                        });
                                    }
                                }
                                st.is_adding_param = false;
                                st.show_param_dropdown = false;
                                cx.notify();
                            });
                        })
                        .child("✓ 确认添加"),
                ),
        )
        // 展开的下拉选项列表
        .children(if show_dd {
            Some(
                div()
                    .id("param_key_dropdown_list")
                    .max_h(px(140.0))
                    .overflow_y_scroll()
                    .bg(hsla(220.0 / 360.0, 0.45, 0.10, 0.98))
                    .border_1()
                    .border_color(hsla(215.0 / 360.0, 0.35, 0.30, 1.0))
                    .rounded_md()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .children(key_suggestions.into_iter().map(|sug| {
                        let model_item = model.clone();
                        let s_str = sug.to_string();
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
                                let val = s_str.clone();
                                model_item.update(cx, |st, cx| {
                                    st.new_param_key = val;
                                    st.show_param_dropdown = false;
                                    cx.notify();
                                });
                            })
                            .child(sug)
                    })),
            )
        } else {
            None
        })
}

fn render_params_table(
    state: &AppState,
    params: &[crate::model::node::PropertyItem],
    actor_idx: usize,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div().w_full().flex().flex_col().child(
        div().flex().items_center().px_4().py_1p5()
            .bg(hsla(220.0 / 360.0, 0.25, 0.10, 1.0))
            .border_b_1().border_color(hsla(220.0 / 360.0, 0.25, 0.16, 1.0))
            .text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(hsla(215.0 / 360.0, 0.20, 0.65, 1.0))
            .child(div().w(px(100.0)).child(state.t("actors_view.th_type")))
            .child(div().w(px(200.0)).child(state.t("actors_view.th_key")))
            .child(div().w(px(160.0)).child(state.t("actors_view.th_val")))
            .child(div().flex_1().child(state.t("actors_view.th_operations"))),
    ).children(params.iter().enumerate().map(|(p_idx, prop)| {
        let model_del = model.clone();
        div().flex().items_center().px_4().py_1p5()
            .border_b_1().border_color(hsla(220.0 / 360.0, 0.25, 0.12, 0.6))
            .hover(|s| s.bg(hsla(215.0 / 360.0, 0.25, 0.15, 0.8)))
            .text_size(px(11.5))
            .child(div().w(px(100.0)).text_color(hsla(200.0 / 360.0, 0.80, 0.70, 1.0)).child(prop.val_type.clone()))
            .child(div().w(px(200.0)).font_weight(FontWeight::SEMIBOLD).child(prop.key.clone()))
            .child(div().w(px(160.0)).text_color(hsla(45.0 / 360.0, 0.90, 0.75, 1.0)).child(prop.val.clone()))
            .child(div().flex_1().child(
                div().w(px(20.0)).h(px(20.0)).rounded_md().hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.9)))
                    .flex().items_center().justify_center().cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                        model_del.update(cx, |st, cx| {
                            if let Some(act) = st.graph.actors.get_mut(actor_idx) {
                                if p_idx < act.params.len() {
                                    act.params.remove(p_idx);
                                }
                            }
                            cx.notify();
                        });
                    }).child("🗑️"),
            ))
    }))
}

