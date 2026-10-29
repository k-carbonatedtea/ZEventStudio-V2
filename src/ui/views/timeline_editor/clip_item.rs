use gpui::*;
use revfl::timeline::Clip;
use crate::model::timeline::{DragMode, DraggingClipState, TimelineSelection};
use crate::ui::state::AppState;

/// 渲染时间线轨道上的单个剪辑动作胶囊 (Clip Card)
#[allow(clippy::too_many_arguments)]
pub fn render_clip_item(
    clip: &Clip,
    clip_idx: usize,
    action_name: String,
    actor_color: Hsla,
    is_selected: bool,
    zoom: f32,
    scroll_x: f32,
    pal: &crate::ui::theme::ThemePalette,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let x = clip.start_time * zoom - scroll_x;
    let width = (clip.duration * zoom).max(28.0);
    let duration_frames = clip.duration.round() as i32;
    let has_params = clip.params.as_ref().is_some_and(|p| !p.data.is_empty());

    let model_select = model.clone();
    let model_resize_left = model.clone();
    let model_resize_right = model.clone();

    let clip_start = clip.start_time;
    let clip_dur = clip.duration;
    let actor_idx = clip.actor.idx as usize;

    // 视口边界快速剔除 (Viewport Culling)
    if x + width < -100.0 || x > 3500.0 {
        return div().into_any_element();
    }

    div()
        .absolute()
        .top(px(4.0))
        .left(px(x))
        .w(px(width))
        .h(px(32.0))
        .rounded_md()
        .bg(if is_selected {
            hsla(actor_color.h, actor_color.s * 0.8, 0.28, 0.95)
        } else {
            hsla(actor_color.h, actor_color.s * 0.6, 0.20, 0.85)
        })
        .border_1()
        .border_color(if is_selected {
            pal.primary_accent
        } else {
            hsla(actor_color.h, actor_color.s, 0.45, 0.70).into()
        })
        .shadow_sm()
        .flex()
        .items_center()
        .justify_between()
        .overflow_hidden()
        // 1. 左侧缩放手柄 (Trim Start)
        .child(
            div()
                .w(px(6.0))
                .h_full()
                .cursor_col_resize()
                .hover(|s| s.bg(pal.primary_accent))
                .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    cx.stop_propagation();
                    let start_mouse_x: f32 = event.position.x.into();
                    model_resize_left.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.selected = TimelineSelection::Clip(clip_idx);
                            ts.dragging_clip = Some(DraggingClipState {
                                clip_idx,
                                drag_mode: DragMode::ResizeStart,
                                initial_start_time: clip_start,
                                initial_duration: clip_dur,
                                initial_mouse_x: start_mouse_x,
                                initial_actor_idx: actor_idx,
                            });
                            cx.notify();
                        }
                    });
                }),
        )
        // 2. 中间信息区域 (动作名、持续时长、参数标记)
        .child(
            div()
                .flex_1()
                .h_full()
                .px_1p5()
                .flex()
                .items_center()
                .justify_between()
                .cursor_grab()
                .overflow_hidden()
                .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    cx.stop_propagation();
                    let start_mouse_x: f32 = event.position.x.into();
                    let m_sel = model_select.clone();
                    m_sel.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.selected = TimelineSelection::Clip(clip_idx);
                            ts.dragging_clip = Some(DraggingClipState {
                                clip_idx,
                                drag_mode: DragMode::Move,
                                initial_start_time: clip_start,
                                initial_duration: clip_dur,
                                initial_mouse_x: start_mouse_x,
                                initial_actor_idx: actor_idx,
                            });
                            cx.notify();
                        }
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .overflow_hidden()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.0))
                                .text_color(if is_selected { pal.text_primary } else { hsla(actor_color.h, 0.90, 0.85, 1.0).into() })
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(if action_name.is_empty() { "Action".to_string() } else { action_name }),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .flex_shrink_0()
                        .children(has_params.then(|| {
                            div()
                                .text_size(px(10.0))
                                .text_color(pal.secondary_accent)
                                .child("⚙️")
                        }))
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(pal.text_secondary)
                                .child(format!("{}f", duration_frames)),
                        ),
                ),
        )
        // 3. 右侧缩放手柄 (Trim End)
        .child(
            div()
                .w(px(6.0))
                .h_full()
                .cursor_col_resize()
                .hover(|s| s.bg(pal.primary_accent))
                .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    cx.stop_propagation();
                    let start_mouse_x: f32 = event.position.x.into();
                    model_resize_right.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.selected = TimelineSelection::Clip(clip_idx);
                            ts.dragging_clip = Some(DraggingClipState {
                                clip_idx,
                                drag_mode: DragMode::ResizeEnd,
                                initial_start_time: clip_start,
                                initial_duration: clip_dur,
                                initial_mouse_x: start_mouse_x,
                                initial_actor_idx: actor_idx,
                            });
                            cx.notify();
                        }
                    });
                }),
        )
        .into_any_element()
}

/// 根据角色索引与角色名映射协调的主题色
pub fn get_actor_accent_color(actor_idx: usize) -> Hsla {
    let hues = [
        200.0, // 遗迹天蓝
        35.0,  // 琥珀金
        145.0, // 翡翠绿
        280.0, // 极光紫
        330.0, // 玫瑰洋红
        15.0,  // 珊瑚朱红
        175.0, // 青碧
        240.0, // 靛蓝
    ];
    let h = hues[actor_idx % hues.len()];
    hsla(h / 360.0, 0.85, 0.60, 1.0)
}
