use gpui::*;
use crate::model::timeline::TimelineSelection;
use crate::ui::state::AppState;

/// 渲染时间线新建角色、剪辑与分镜弹窗
pub fn render_timeline_modals(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let ts = match state.active_timeline_state() {
        Some(ts) => ts,
        None => return div().into_any_element(),
    };

    if ts.modal_add_actor_open {
        return render_add_actor_modal(state, model).into_any_element();
    }
    if ts.modal_add_clip_open {
        return render_add_clip_modal(state, model).into_any_element();
    }
    if ts.modal_add_cut_open {
        return render_add_cut_modal(state, model).into_any_element();
    }

    div().into_any_element()
}

/// 渲染「添加角色」弹窗与 BotW 常用预设库
fn render_add_actor_modal(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let m_close = model.clone();

    let presets = [
        ("GameRomCamera", "", vec!["Demo_CameraAnim", "Demo_PermitGfxNear", "Demo_Focus"]),
        ("GameROMPlayer", "", vec!["Demo_PlayerRailMove", "Demo_PlayerDestinationMove", "Demo_PlayerDestinationTurn", "Demo_LookAtObject", "Demo_LookAtTheFront"]),
        ("EventSystemActor", "", vec!["Demo_PlayUiScreen", "Demo_FlagON", "Demo_FlagOFF", "Demo_RecoverPlayerEnergy", "Demo_WaitFrame"]),
        ("SceneSoundCtrlTag", "", vec!["Demo_Ctrl", "Demo_BgmCtrl", "Demo_SeCtrl"]),
        ("TerrainCalcCenterTag", "", vec!["Demo_TerrainCalcCenter"]),
        ("EventMessageTransmitter1", "", vec!["Demo_Msg2CameraKeepState"]),
        ("WorldManagerControl", "", vec!["Demo_SetWeather", "Demo_SetTime"]),
        ("Fader", "", vec!["Demo_FadeIn", "Demo_FadeOut"]),
    ];

    div()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(460.0))
                .rounded_lg()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.primary_accent)
                .shadow_lg()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                // 标题与关闭
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("timeline.modal_add_actor_title")),
                        )
                        .child(
                            div()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(rgb(0xef4444)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m_close.update(cx, |st, cx| {
                                        if let Some(ts) = st.active_timeline_state_mut() {
                                            ts.modal_add_actor_open = false;
                                            cx.notify();
                                        }
                                    });
                                })
                                .child("✕"),
                        ),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(pal.secondary_accent)
                        .child(state.t("timeline.preset_actors")),
                )
                // 预设列表
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .max_h(px(260.0))
                        .id("preset_actors_scroll")
                        .overflow_y_scroll()
                        .children(presets.into_iter().map(|(name, sec, acts)| {
                            let m_pick = model.clone();
                            let n = name.to_string();
                            let s = sec.to_string();
                            let actions: Vec<String> = acts.into_iter().map(|a| a.to_string()).collect();
                            let acts_str = actions.join(", ");

                            div()
                                .p_2()
                                .rounded_sm()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.panel_border)
                                .cursor_pointer()
                                .hover(|st| st.bg(pal.card_hover).border_color(pal.primary_accent))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let n_clone = n.clone();
                                    let s_clone = s.clone();
                                    let a_clone = actions.clone();
                                    m_pick.update(cx, |st, cx| {
                                        if let Some(ts) = st.active_timeline_state_mut() {
                                            let new_idx = crate::model::timeline::add_actor_to_timeline(
                                                &mut ts.timeline,
                                                n_clone,
                                                s_clone,
                                                a_clone,
                                            );
                                            ts.selected = TimelineSelection::Actor(new_idx);
                                            ts.modal_add_actor_open = false;
                                            st.sync_active_tab_modified();
                                            cx.notify();
                                        }
                                    });
                                })
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(div().font_weight(FontWeight::BOLD).text_size(px(11.5)).text_color(pal.text_primary).child(name))
                                        .child(div().text_size(px(10.0)).text_color(pal.text_secondary).child(acts_str)),
                                )
                        })),
                ),
        )
}

/// 渲染「新建剪辑」弹窗
fn render_add_clip_modal(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let ts = match state.active_timeline_state() {
        Some(ts) => ts,
        None => return div(),
    };

    let actors = ts.timeline.actors.clone();
    let current_frame = ts.current_frame;
    let selected = ts.selected;
    let default_actor_idx = match selected {
        TimelineSelection::Actor(idx) => idx,
        TimelineSelection::Clip(idx) => ts.timeline.clips.get(idx).map(|c| c.actor.idx as usize).unwrap_or(0),
        _ => 0,
    };

    let m_close = model.clone();
    let m_confirm = model.clone();

    div()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(380.0))
                .rounded_lg()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.primary_accent)
                .shadow_lg()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                // 标题与关闭
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("timeline.modal_add_clip_title")),
                        )
                        .child(
                            div()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(rgb(0xef4444)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m_close.update(cx, |st, cx| {
                                        if let Some(ts) = st.active_timeline_state_mut() {
                                            ts.modal_add_clip_open = false;
                                            cx.notify();
                                        }
                                    });
                                })
                                .child("✕"),
                        ),
                )
                // 选择角色与动作
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(actors.iter().enumerate().map(|(idx, actor)| {
                            let act_name = actor.identifier.name.clone();
                            let is_chosen = idx == default_actor_idx;
                            let actions: Vec<String> = actor.actions.iter().map(|a| a.0.clone()).collect();
                            let m_sel = m_confirm.clone();

                            div()
                                .p_2()
                                .rounded_sm()
                                .border_1()
                                .border_color(if is_chosen { pal.primary_accent } else { pal.panel_border })
                                .bg(if is_chosen { pal.card_hover } else { pal.card_surface })
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m_sel.update(cx, |st, cx| {
                                        if let Some(ts) = st.active_timeline_state_mut() {
                                            let new_clip_idx = crate::model::timeline::add_clip_to_timeline(
                                                &mut ts.timeline,
                                                idx,
                                                0,
                                                current_frame,
                                                300.0,
                                            );
                                            ts.selected = TimelineSelection::Clip(new_clip_idx);
                                            ts.modal_add_clip_open = false;
                                            st.sync_active_tab_modified();
                                            cx.notify();
                                        }
                                    });
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(div().font_weight(FontWeight::BOLD).text_size(px(11.5)).text_color(pal.text_primary).child(act_name))
                                        .child(div().text_size(px(10.0)).text_color(pal.secondary_accent).child(format!("{}动作", actions.len()))),
                                )
                        })),
                ),
        )
}

/// 渲染「新建镜头」弹窗
fn render_add_cut_modal(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let ts = match state.active_timeline_state() {
        Some(ts) => ts,
        None => return div(),
    };

    let next_cut_num = ts.timeline.cuts.len();
    let current_frame = ts.current_frame;
    let m_close = model.clone();
    let m_confirm = model.clone();

    div()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(320.0))
                .rounded_lg()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.secondary_accent)
                .shadow_lg()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .child(state.t("timeline.modal_add_cut_title")),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(pal.text_secondary)
                        .child(format!("镜头标识: cut{} (起始: {}f)", next_cut_num, current_frame as i32)),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .h(px(28.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_sm()
                                .cursor_pointer()
                                .bg(pal.card_surface)
                                .text_size(px(11.0))
                                .text_color(pal.text_primary)
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m_close.update(cx, |st, cx| {
                                        if let Some(ts) = st.active_timeline_state_mut() {
                                            ts.modal_add_cut_open = false;
                                            cx.notify();
                                        }
                                    });
                                })
                                .child(state.t("timeline.cancel")),
                        )
                        .child(
                            div()
                                .flex_1()
                                .h(px(28.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_sm()
                                .cursor_pointer()
                                .bg(pal.secondary_accent)
                                .text_color(pal.app_bg)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.0))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m_confirm.update(cx, |st, cx| {
                                        if let Some(ts) = st.active_timeline_state_mut() {
                                            let name = format!("cut{}", next_cut_num);
                                            let idx = crate::model::timeline::add_cut_to_timeline(&mut ts.timeline, name, current_frame);
                                            ts.selected = TimelineSelection::Cut(idx);
                                            ts.modal_add_cut_open = false;
                                            st.sync_active_tab_modified();
                                            cx.notify();
                                        }
                                    });
                                })
                                .child(state.t("timeline.confirm")),
                        ),
                ),
        )
}
