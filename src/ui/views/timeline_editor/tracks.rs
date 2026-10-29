use gpui::*;
use crate::model::timeline::{DragMode, TimelineSelection};
use crate::ui::state::AppState;
use revfl::timeline::Clip;
use super::clip_item::{get_actor_accent_color, render_clip_item};

/// 渲染时间线多角色多轨道主体视图 (Track List & Lanes)
pub fn render_timeline_tracks(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let ts = match state.active_timeline_state() {
        Some(ts) => ts,
        None => return div().into_any_element(),
    };

    let header_width = ts.header_width;
    let zoom = ts.zoom;
    let scroll_x = ts.scroll_x;
    let current_frame = ts.current_frame;
    let actors = ts.timeline.actors.clone();
    let clips = ts.timeline.clips.clone();
    let selected = ts.selected;
    let dragging = ts.dragging_clip.clone();
    let model_lane_drag = model.clone();
    let model_lane_up = model.clone();
    let model_actor_click = model.clone();
    let model_add_clip = model.clone();

    // 播放游标在轨道视口内的水平坐标
    let playhead_x = current_frame * zoom - scroll_x;

    div()
        .id("timeline-tracks-container")
        .flex_1()
        .w_full()
        .flex()
        .overflow_hidden()
        // 处理正在进行的剪辑拖拽移动与拉伸 (Mouse Move & Mouse Up)
        .on_mouse_move(move |event, _, cx| {
            if let Some(ref drag) = dragging {
                let mx: f32 = event.position.x.into();
                let delta_x = mx - drag.initial_mouse_x;
                let delta_frames = delta_x / zoom;

                model_lane_drag.update(cx, |st, cx| {
                    if let Some(ts) = st.active_timeline_state_mut() {
                        let c_idx = drag.clip_idx;
                        match drag.drag_mode {
                            DragMode::Move => {
                                let new_start = (drag.initial_start_time + delta_frames).max(0.0);
                                crate::model::timeline::move_clip_start(&mut ts.timeline, c_idx, new_start);
                            }
                            DragMode::ResizeStart => {
                                let new_start = (drag.initial_start_time + delta_frames).max(0.0);
                                let new_dur = (drag.initial_duration - delta_frames).max(1.0);
                                crate::model::timeline::resize_clip(&mut ts.timeline, c_idx, new_start, new_dur);
                            }
                            DragMode::ResizeEnd => {
                                let new_dur = (drag.initial_duration + delta_frames).max(1.0);
                                crate::model::timeline::resize_clip(&mut ts.timeline, c_idx, drag.initial_start_time, new_dur);
                            }
                        }
                        cx.notify();
                    }
                });
            }
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            model_lane_up.update(cx, |st, cx| {
                let mut should_sync = false;
                if let Some(ts) = st.active_timeline_state_mut() {
                    if ts.dragging_clip.is_some() {
                        ts.dragging_clip = None;
                        should_sync = true;
                    }
                    if ts.is_scrubbing {
                        ts.is_scrubbing = false;
                    }
                }
                if should_sync {
                    st.sync_active_tab_modified();
                }
                cx.notify();
            });
        })
        // 1. 左侧角色轨道头部列表 (Track Headers)
        .child(
            div()
                .w(px(header_width))
                .h_full()
                .flex_shrink_0()
                .bg(pal.panel_surface)
                .border_r_1()
                .border_color(pal.panel_border)
                .id("timeline_tracks_header_scroll")
                .overflow_y_scroll()
                .children(actors.iter().enumerate().map(|(idx, actor)| {
                    let is_actor_selected = selected == TimelineSelection::Actor(idx);
                    let accent_color = get_actor_accent_color(idx);
                    let act_name = actor.identifier.name.clone();
                    let sec_name = actor.identifier.sub_name.clone();
                    let action_count = actor.actions.len();

                    let m_act = model_actor_click.clone();
                    let m_add = model_add_clip.clone();

                    div()
                        .h(px(40.0))
                        .w_full()
                        .px_3()
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_b_1()
                        .border_color(pal.panel_border)
                        .cursor_pointer()
                        .bg(if is_actor_selected { pal.card_hover } else { pal.card_surface })
                        .border_l_4()
                        .border_color(accent_color)
                        .hover(|s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m_act.update(cx, |st, cx| {
                                if let Some(ts) = st.active_timeline_state_mut() {
                                    ts.selected = TimelineSelection::Actor(idx);
                                    cx.notify();
                                }
                            });
                        })
                        // 角色名称与副标识
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(11.5))
                                        .text_color(if is_actor_selected { pal.primary_accent } else { pal.text_primary })
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .child(act_name),
                                )
                                .children((!sec_name.is_empty()).then(|| {
                                    div()
                                        .text_size(px(9.5))
                                        .text_color(pal.text_secondary)
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .child(sec_name)
                                })),
                        )
                        // 操作区：动作数徽章与加号
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(pal.app_bg)
                                        .text_size(px(9.5))
                                        .text_color(pal.text_secondary)
                                        .child(format!("{}动作", action_count)),
                                )
                                .child(
                                    div()
                                        .w(px(20.0))
                                        .h(px(20.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_sm()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(pal.primary_accent).text_color(pal.app_bg))
                                        .text_color(pal.text_secondary)
                                        .text_size(px(11.0))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            cx.stop_propagation();
                                            m_add.update(cx, |st, cx| {
                                                if let Some(ts) = st.active_timeline_state_mut() {
                                                    ts.selected = TimelineSelection::Actor(idx);
                                                    ts.modal_add_clip_open = true;
                                                    cx.notify();
                                                }
                                            });
                                        })
                                        .child("+"),
                                ),
                        )
                })),
        )
        // 2. 右侧时间线轨道区域 (Track Lanes)
        .child(
            div()
                .flex_1()
                .h_full()
                .relative()
                .overflow_hidden()
                .bg(pal.app_bg)
                // 轨道行与剪辑容器
                .child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .flex_col()
                        .children(actors.iter().enumerate().map(|(actor_idx, actor)| {
                            let accent_color = get_actor_accent_color(actor_idx);
                            let lane_clips: Vec<(usize, &Clip)> = clips
                                .iter()
                                .enumerate()
                                .filter(|(_, c)| c.actor.idx as usize == actor_idx)
                                .collect();

                            div()
                                .h(px(40.0))
                                .w_full()
                                .relative()
                                .border_b_1()
                                .border_color(pal.panel_border)
                                .bg(if actor_idx % 2 == 0 { pal.app_bg } else { pal.card_surface })
                                // 渲染属于该角色的所有剪辑
                                .children(lane_clips.into_iter().map(|(clip_idx, clip)| {
                                    let action_idx = clip.actor_action.idx as usize;
                                    let action_name = actor.actions.get(action_idx)
                                        .map(|a| a.0.clone())
                                        .unwrap_or_else(|| format!("Action_{}", action_idx));
                                    let is_clip_selected = selected == TimelineSelection::Clip(clip_idx);

                                    render_clip_item(
                                        clip,
                                        clip_idx,
                                        action_name,
                                        accent_color,
                                        is_clip_selected,
                                        zoom,
                                        scroll_x,
                                        &pal,
                                        model,
                                    )
                                }))
                        })),
                )
                // 贯穿所有轨道的红色/青色播放针游标竖线 (Playhead Line)
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(px(playhead_x))
                        .w(px(2.0))
                        .bg(pal.primary_accent)
                        .shadow_sm(),
                ),
        )
        .into_any_element()
}
