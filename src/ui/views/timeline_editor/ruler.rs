use gpui::*;
use crate::model::timeline::TimelineSelection;
use crate::ui::state::AppState;

/// 渲染时间线顶部时间刻度尺与分镜标记栏
pub fn render_timeline_ruler(
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
    let total_duration = ts.timeline.duration.max(10.0);
    let current_frame = ts.current_frame;
    let cuts = ts.timeline.cuts.clone();
    let selected = ts.selected;

    // 根据 zoom 动态计算主刻度步长 (确保刻度间距不低于 60px)
    let step_frames = if zoom >= 4.0 {
        10.0
    } else if zoom >= 1.5 {
        30.0
    } else if zoom >= 0.75 {
        60.0
    } else if zoom >= 0.3 {
        120.0
    } else {
        300.0
    };

    let model_scrub = model.clone();
    let model_cut_click = model.clone();

    // 播放游标在刻度尺视口内的水平坐标
    let playhead_x = current_frame * zoom - scroll_x;

    div()
        .h(px(32.0))
        .w_full()
        .flex()
        .bg(pal.panel_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        // 1. 左侧表头标题：镜头分镜 (Cuts)
        .child(
            div()
                .w(px(header_width))
                .h_full()
                .flex_shrink_0()
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_r_1()
                .border_color(pal.panel_border)
                .bg(pal.card_surface)
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(11.5))
                        .text_color(pal.text_primary)
                        .child(state.t("timeline.cuts_track")),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .child(format!("{} 镜头", cuts.len())),
                ),
        )
        // 2. 右侧时间刻度画布与镜头标记
        .child(
            div()
                .id("timeline-ruler-canvas")
                .flex_1()
                .h_full()
                .relative()
                .overflow_hidden()
                .cursor_crosshair()
                // 点击或划选时间轴调整播放游标
                .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    let mouse_x: f32 = event.position.x.into();
                    model_scrub.update(cx, |st, cx| {
                        let origin_x = (if st.sidebar_open { 280.0 } else { 0.0 }) + 220.0;
                        if let Some(ts) = st.active_timeline_state_mut() {
                            let rel_x = (mouse_x - origin_x).max(0.0);
                            let target_frame = ((rel_x + ts.scroll_x) / ts.zoom).clamp(0.0, ts.timeline.duration.max(100.0));
                            ts.current_frame = target_frame;
                            ts.is_scrubbing = true;
                            cx.notify();
                        }
                    });
                })
                // 绘制刻度线与时间标签
                .children(render_ruler_ticks(zoom, scroll_x, total_duration, step_frames, &pal))
                // 绘制镜头标记 (Cuts)
                .children(cuts.into_iter().enumerate().map(|(idx, cut)| {
                    let cut_x = cut.start_time * zoom - scroll_x;
                    let is_cut_selected = selected == TimelineSelection::Cut(idx);
                    let cut_name = cut.name.clone();
                    let m_cut = model_cut_click.clone();

                    div()
                        .absolute()
                        .top(px(2.0))
                        .left(px(cut_x))
                        .h(px(20.0))
                        .px_1p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_1()
                        .bg(if is_cut_selected { pal.secondary_accent } else { pal.card_surface })
                        .border_1()
                        .border_color(if is_cut_selected { pal.secondary_accent } else { pal.panel_border })
                        .text_color(if is_cut_selected { pal.app_bg } else { pal.text_primary })
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .hover(move |s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m_cut.update(cx, |st, cx| {
                                if let Some(ts) = st.active_timeline_state_mut() {
                                    ts.selected = TimelineSelection::Cut(idx);
                                    ts.current_frame = cut.start_time;
                                    cx.notify();
                                }
                            });
                        })
                        .child("🚩")
                        .child(if cut_name.is_empty() { format!("cut{}", idx) } else { cut_name })
                }))
                // 绘制播放游标头部 (Playhead Cursor)
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left(px(playhead_x - 5.0))
                        .w(px(11.0))
                        .h(px(14.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.primary_accent)
                                .child("▼"),
                        ),
                ),
        )
        .into_any_element()
}

/// 批量生成刻度线与时间标签元素
fn render_ruler_ticks(
    zoom: f32,
    scroll_x: f32,
    total_duration: f32,
    step_frames: f32,
    pal: &crate::ui::theme::ThemePalette,
) -> Vec<impl IntoElement> {
    let mut elements = Vec::new();
    let num_ticks = (total_duration / step_frames).ceil() as i32 + 1;

    for i in 0..=num_ticks {
        let frame = i as f32 * step_frames;
        if frame > total_duration + step_frames {
            break;
        }
        let x = frame * zoom - scroll_x;
        if !(-50.0..=3000.0).contains(&x) {
            continue; // 视口裁剪 Viewport Culling
        }

        let secs = frame / 30.0;
        let time_str = format!("{}:{:02}", (secs / 60.0).floor() as i32, (secs % 60.0).floor() as i32);

        elements.push(
            div()
                .absolute()
                .left(px(x))
                .bottom_0()
                .flex()
                .flex_col()
                .items_start()
                .child(
                    div()
                        .w(px(1.0))
                        .h(px(8.0))
                        .bg(pal.panel_border),
                )
                .child(
                    div()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(format!("{}f ({})", frame as i32, time_str)),
                ),
        );
    }

    elements
}
