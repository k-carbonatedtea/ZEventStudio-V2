use gpui::*;
use crate::model::timeline::param_editor::{get_container_param_list, remove_container_param};
use crate::model::timeline::TimelineSelection;
use crate::ui::state::AppState;

/// 渲染时间线右侧属性检查器面板 (Inspector)
pub fn render_timeline_inspector(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let ts = match state.active_timeline_state() {
        Some(ts) => ts,
        None => return div().into_any_element(),
    };

    if ts.inspector_collapsed {
        return div().into_any_element();
    }

    let selected = ts.selected;
    let model_content = model.clone();

    div()
        .w(px(280.0))
        .h_full()
        .flex_shrink_0()
        .bg(pal.panel_surface)
        .border_l_1()
        .border_color(pal.panel_border)
        .flex()
        .flex_col()
        .overflow_hidden()
        // 1. 顶部面板标题栏
        .child(
            div()
                .h(px(36.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(pal.panel_border)
                .bg(pal.card_surface)
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .child(state.t("timeline.inspector")),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(match selected {
                            TimelineSelection::Clip(_) => state.t("timeline.clip_props"),
                            TimelineSelection::Actor(_) => state.t("timeline.actor_props"),
                            TimelineSelection::Cut(_) => state.t("timeline.cut_props"),
                            _ => state.t("timeline.timeline_props"),
                        }),
                ),
        )
        // 2. 检查器内容区
        .child(
            div()
                .flex_1()
                .p_3()
                .flex()
                .flex_col()
                .id("timeline_inspector_scroll")
                .overflow_y_scroll()
                .child(match selected {
                    TimelineSelection::Clip(c_idx) => render_clip_inspector(ts, c_idx, &pal, state, &model_content).into_any_element(),
                    TimelineSelection::Actor(a_idx) => render_actor_inspector(ts, a_idx, &pal, state, &model_content).into_any_element(),
                    TimelineSelection::Cut(k_idx) => render_cut_inspector(ts, k_idx, &pal, state, &model_content).into_any_element(),
                    _ => render_global_inspector(ts, &pal, state, &model_content).into_any_element(),
                }),
        )
        .into_any_element()
}

/// 渲染选中剪辑属性
fn render_clip_inspector(
    ts: &crate::model::TimelineDocumentState,
    clip_idx: usize,
    pal: &crate::ui::theme::ThemePalette,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let clip = match ts.timeline.clips.get(clip_idx) {
        Some(c) => c,
        None => return div().child("剪辑已不存在").into_any_element(),
    };

    let actor_idx = clip.actor.idx as usize;
    let action_idx = clip.actor_action.idx as usize;
    let actor_name = ts.timeline.actors.get(actor_idx)
        .map(|a| a.identifier.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let action_name = ts.timeline.actors.get(actor_idx)
        .and_then(|a| a.actions.get(action_idx))
        .map(|a| a.0.clone())
        .unwrap_or_else(|| format!("Action_{}", action_idx));

    let start_frame = clip.start_time.round() as i32;
    let dur_frames = clip.duration.round() as i32;
    let end_frame = (clip.start_time + clip.duration).round() as i32;

    let param_items = clip.params.as_ref().map(get_container_param_list).unwrap_or_default();

    let m_del = model.clone();
    let m_dup = model.clone();
    let m_del_param = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(render_info_row(state.t("timeline.actor"), &actor_name, pal))
        .child(render_info_row(state.t("timeline.action"), &action_name, pal))
        .child(render_info_row(state.t("timeline.start_time"), &format!("{}f", start_frame), pal))
        .child(render_info_row(state.t("timeline.clip_duration"), &format!("{}f", dur_frames), pal))
        .child(render_info_row(state.t("timeline.end_time"), &format!("{}f", end_frame), pal))
        // 参数卡片区
        .child(
            div()
                .mt_1()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(11.0))
                        .text_color(pal.secondary_accent)
                        .child(format!("{} ({})", state.t("timeline.params"), param_items.len())),
                )
                .children(param_items.into_iter().map(|item| {
                    let k = item.key.clone();
                    let m_dp = m_del_param.clone();
                    div()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(div().font_weight(FontWeight::BOLD).text_size(px(10.5)).text_color(pal.text_primary).child(item.key))
                                .child(div().text_size(px(9.5)).text_color(pal.primary_accent).child(format!("{}: {}", item.type_name, item.value_str))),
                        )
                        .child(
                            div()
                                .cursor_pointer()
                                .text_size(px(10.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(rgb(0xef4444)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let key_to_del = k.clone();
                                    m_dp.update(cx, |st, cx| {
                                        if let Some(ts) = st.active_timeline_state_mut() {
                                            if let Some(clip) = ts.timeline.clips.get_mut(clip_idx) {
                                                if let Some(params) = &mut clip.params {
                                                    remove_container_param(params, &key_to_del);
                                                }
                                            }
                                            st.sync_active_tab_modified();
                                            cx.notify();
                                        }
                                    });
                                })
                                .child("✕"),
                        )
                })),
        )
        // 操作按钮行
        .child(
            div()
                .mt_2()
                .flex()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .h(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_sm()
                        .cursor_pointer()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(11.0))
                        .text_color(pal.text_primary)
                        .hover(|s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m_dup.update(cx, |st, cx| {
                                if let Some(ts) = st.active_timeline_state_mut() {
                                    if let Some(new_idx) = crate::model::timeline::duplicate_clip_in_timeline(&mut ts.timeline, clip_idx) {
                                        ts.selected = TimelineSelection::Clip(new_idx);
                                    }
                                    st.sync_active_tab_modified();
                                    cx.notify();
                                }
                            });
                        })
                        .child(state.t("timeline.duplicate_clip")),
                )
                .child(
                    div()
                        .flex_1()
                        .h(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_sm()
                        .cursor_pointer()
                        .bg(rgb(0xef4444))
                        .text_size(px(11.0))
                        .text_color(pal.app_bg)
                        .font_weight(FontWeight::BOLD)
                        .hover(|s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m_del.update(cx, |st, cx| {
                                if let Some(ts) = st.active_timeline_state_mut() {
                                    crate::model::timeline::remove_clip_from_timeline(&mut ts.timeline, clip_idx);
                                    ts.selected = TimelineSelection::None;
                                    st.sync_active_tab_modified();
                                    cx.notify();
                                }
                            });
                        })
                        .child(state.t("timeline.delete_clip")),
                ),
        )
        .into_any_element()
}

/// 渲染选中角色属性
fn render_actor_inspector(
    ts: &crate::model::TimelineDocumentState,
    actor_idx: usize,
    pal: &crate::ui::theme::ThemePalette,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let actor = match ts.timeline.actors.get(actor_idx) {
        Some(a) => a,
        None => return div().child("角色已不存在").into_any_element(),
    };

    let act_name = actor.identifier.name.clone();
    let sec_name = actor.identifier.sub_name.clone();
    let actions: Vec<String> = actor.actions.iter().map(|a| a.0.clone()).collect();
    let m_del = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(render_info_row(state.t("timeline.actor_name"), &act_name, pal))
        .child(render_info_row(state.t("timeline.secondary_name"), &sec_name, pal))
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(11.0))
                .text_color(pal.secondary_accent)
                .child(format!("{} ({})", state.t("timeline.registered_actions"), actions.len())),
        )
        .children(actions.into_iter().map(|act| {
            div()
                .px_2()
                .py_1()
                .rounded_sm()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.panel_border)
                .text_size(px(10.5))
                .text_color(pal.text_primary)
                .child(act)
        }))
        .child(
            div()
                .mt_3()
                .h(px(26.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded_sm()
                .cursor_pointer()
                .bg(rgb(0xef4444))
                .text_size(px(11.0))
                .text_color(pal.app_bg)
                .font_weight(FontWeight::BOLD)
                .hover(|s| s.opacity(0.85))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    m_del.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            crate::model::timeline::remove_actor_from_timeline(&mut ts.timeline, actor_idx);
                            ts.selected = TimelineSelection::None;
                            st.sync_active_tab_modified();
                            cx.notify();
                        }
                    });
                })
                .child(state.t("timeline.delete_actor")),
        )
        .into_any_element()
}

/// 渲染镜头分镜属性
fn render_cut_inspector(
    ts: &crate::model::TimelineDocumentState,
    cut_idx: usize,
    pal: &crate::ui::theme::ThemePalette,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let cut = match ts.timeline.cuts.get(cut_idx) {
        Some(c) => c,
        None => return div().child("镜头已不存在").into_any_element(),
    };

    let cut_name = cut.name.clone();
    let start_time = cut.start_time.round() as i32;

    let m_del = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(render_info_row(state.t("timeline.cut_name"), &cut_name, pal))
        .child(render_info_row(state.t("timeline.start_time"), &format!("{}f", start_time), pal))
        .child(
            div()
                .mt_3()
                .h(px(26.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded_sm()
                .cursor_pointer()
                .bg(rgb(0xef4444))
                .text_size(px(11.0))
                .text_color(pal.app_bg)
                .font_weight(FontWeight::BOLD)
                .hover(|s| s.opacity(0.85))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    m_del.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            crate::model::timeline::remove_cut_from_timeline(&mut ts.timeline, cut_idx);
                            ts.selected = TimelineSelection::None;
                            st.sync_active_tab_modified();
                            cx.notify();
                        }
                    });
                })
                .child(state.t("timeline.delete_cut")),
        )
        .into_any_element()
}

/// 渲染全局时间线概览
fn render_global_inspector(
    ts: &crate::model::TimelineDocumentState,
    pal: &crate::ui::theme::ThemePalette,
    state: &AppState,
    _model: &Entity<AppState>,
) -> impl IntoElement {
    let dur = ts.timeline.duration.round() as i32;
    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(render_info_row(state.t("timeline.duration"), &format!("{} 帧", dur), pal))
        .child(render_info_row(state.t("timeline.total_actors"), &format!("{}", ts.timeline.actors.len()), pal))
        .child(render_info_row(state.t("timeline.total_clips"), &format!("{}", ts.timeline.clips.len()), pal))
        .child(render_info_row(state.t("timeline.total_cuts"), &format!("{}", ts.timeline.cuts.len()), pal))
}

fn render_info_row(label: &str, val: &str, pal: &crate::ui::theme::ThemePalette) -> impl IntoElement {
    div()
        .flex().items_center().justify_between().py_1().border_b_1().border_color(pal.panel_border)
        .child(div().text_size(px(11.0)).text_color(pal.text_secondary).child(label.to_string()))
        .child(div().text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(pal.text_primary).child(val.to_string()))
}
