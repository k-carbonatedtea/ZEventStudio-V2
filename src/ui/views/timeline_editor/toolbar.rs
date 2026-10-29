use gpui::*;
use crate::ui::state::{AppState, AppWorkspaceTab};

/// 渲染时间线顶部控制与回放工具栏
pub fn render_timeline_toolbar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let ts = match state.active_timeline_state() {
        Some(ts) => ts,
        None => return div().into_any_element(),
    };

    let timeline_name = ts.timeline.name.clone();
    let total_duration = ts.timeline.duration;
    let current_frame = ts.current_frame.round() as i32;
    let is_playing = ts.is_playing;
    let current_fps = ts.fps;
    let current_zoom = ts.zoom;
    let inspector_collapsed = ts.inspector_collapsed;

    // 计算当前时间对应秒数格式 MM:SS.ff
    let total_secs = total_duration / current_fps;
    let cur_secs = ts.current_frame / current_fps;
    let time_display = format!(
        "{:02}:{:05.2} / {:02}:{:05.2}",
        (cur_secs / 60.0).floor() as i32,
        cur_secs % 60.0,
        (total_secs / 60.0).floor() as i32,
        total_secs % 60.0
    );

    let model_play = model.clone();
    let model_stop = model.clone();
    let model_prev = model.clone();
    let model_next = model.clone();
    let model_fps = model.clone();
    let model_zoom_in = model.clone();
    let model_zoom_out = model.clone();
    let model_zoom_fit = model.clone();
    let model_add_actor = model.clone();
    let model_add_clip = model.clone();
    let model_add_cut = model.clone();
    let model_save = model.clone();
    let model_toggle_code = model.clone();
    let model_toggle_inspector = model.clone();

    div()
        .h(px(42.0))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .bg(pal.panel_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        // 1. 左侧：时间线名称与时长元数据
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(div().text_size(px(14.0)).child("🎬"))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(if timeline_name.is_empty() { "Untitled".to_string() } else { timeline_name }),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(11.0))
                        .text_color(pal.secondary_accent)
                        .child(format!("{} 帧 ({:.1}s)", total_duration.round() as i32, total_secs)),
                )
                // FPS 切换按钮
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .hover(move |s| s.bg(pal.card_hover).text_color(pal.primary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_fps.update(cx, |st, cx| {
                                if let Some(ts) = st.active_timeline_state_mut() {
                                    ts.fps = if ts.fps == 30.0 { 60.0 } else { 30.0 };
                                    cx.notify();
                                }
                            });
                        })
                        .child(format!("{:.0} FPS", current_fps)),
                ),
        )
        // 2. 中间：回放播放控制器
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                // 归零 ⏮
                .child(render_tool_btn("⏮", state.t("timeline.stop"), &pal, move |cx| {
                    model_stop.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.stop_and_rewind();
                            cx.notify();
                        }
                    });
                }))
                // 上一帧 ◀
                .child(render_tool_btn("◀", state.t("timeline.step_prev"), &pal, move |cx| {
                    model_prev.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.step_frame(-1.0);
                            cx.notify();
                        }
                    });
                }))
                // 播放 / 暂停 ▶ / ⏸
                .child(
                    div()
                        .w(px(32.0))
                        .h(px(28.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .cursor_pointer()
                        .bg(if is_playing { pal.secondary_accent } else { pal.primary_accent })
                        .text_color(pal.app_bg)
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .hover(move |s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_play.update(cx, |st, cx| {
                                if let Some(ts) = st.active_timeline_state_mut() {
                                    ts.toggle_play();
                                    cx.notify();
                                }
                            });
                        })
                        .child(if is_playing { "⏸" } else { "▶" }),
                )
                // 下一帧 ▶
                .child(render_tool_btn("▶", state.t("timeline.step_next"), &pal, move |cx| {
                    model_next.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.step_frame(1.0);
                            cx.notify();
                        }
                    });
                }))
                // 帧数显示与时间戳指示器
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.0))
                                .text_color(pal.primary_accent)
                                .child(format!("{:4}", current_frame)),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(format!("/ {} ({})", total_duration.round() as i32, time_display)),
                        ),
                ),
        )
        // 3. 右侧：视口缩放与新建操作
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                // 缩小 🔍-
                .child(render_tool_btn("➖", state.t("timeline.zoom"), &pal, move |cx| {
                    model_zoom_out.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.zoom = (ts.zoom * 0.8).max(0.2);
                            cx.notify();
                        }
                    });
                }))
                // 放大 🔍+
                .child(render_tool_btn("➕", state.t("timeline.zoom"), &pal, move |cx| {
                    model_zoom_in.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.zoom = (ts.zoom * 1.25).min(15.0);
                            cx.notify();
                        }
                    });
                }))
                // 适应视图 ↔️
                .child(render_tool_btn("↔️", state.t("timeline.zoom_fit"), &pal, move |cx| {
                    model_zoom_fit.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            let dur = ts.timeline.duration.max(100.0);
                            ts.zoom = (900.0 / dur).clamp(0.2, 5.0);
                            ts.scroll_x = 0.0;
                            cx.notify();
                        }
                    });
                }))
                // 缩放等级提示
                .child(
                    div()
                        .w(px(38.0))
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .child(format!("{:.1}x", current_zoom)),
                )
                .child(div().w(px(1.0)).h(px(16.0)).bg(pal.panel_border))
                // ➕ 角色按钮
                .child(render_pill_action_btn("+ 角色", state.t("timeline.add_actor"), &pal, move |cx| {
                    model_add_actor.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.modal_add_actor_open = true;
                            cx.notify();
                        }
                    });
                }))
                // ➕ 剪辑按钮
                .child(render_pill_action_btn("+ 剪辑", state.t("timeline.add_clip"), &pal, move |cx| {
                    model_add_clip.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.modal_add_clip_open = true;
                            cx.notify();
                        }
                    });
                }))
                // ➕ 镜头按钮
                .child(render_pill_action_btn("+ 镜头", state.t("timeline.add_cut"), &pal, move |cx| {
                    model_add_cut.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.modal_add_cut_open = true;
                            cx.notify();
                        }
                    });
                }))
                // 💾 保存按钮
                .child(render_pill_action_btn("💾 保存", state.t("timeline.save"), &pal, move |cx| {
                    model_save.update(cx, |st, cx| {
                        if let Some(tab_id) = st.active_tab_id.clone() {
                            st.save_document_tab(&tab_id);
                            cx.notify();
                        }
                    });
                }))
                // ⇄ 源码同步切换
                .child(render_tool_btn("⇄", state.t("timeline.source"), &pal, move |cx| {
                    model_toggle_code.update(cx, |st, cx| {
                        st.switch_workspace_tab(AppWorkspaceTab::CodeEditor);
                        cx.notify();
                    });
                }))
                // 属性侧边栏开关
                .child(render_tool_btn(
                    if inspector_collapsed { "◨" } else { "◧" },
                    state.t("timeline.inspector"),
                    &pal,
                    move |cx| {
                        model_toggle_inspector.update(cx, |st, cx| {
                            if let Some(ts) = st.active_timeline_state_mut() {
                                ts.inspector_collapsed = !ts.inspector_collapsed;
                                cx.notify();
                            }
                        });
                    },
                )),
        )
        .into_any_element()
}

fn render_tool_btn<F: Fn(&mut App) + 'static>(
    icon: &'static str,
    tooltip: &'static str,
    pal: &crate::ui::theme::ThemePalette,
    on_click: F,
) -> impl IntoElement {
    let _ = tooltip;
    let pal_hover = pal.card_hover;
    let pal_text_hover = pal.primary_accent;
    div()
        .w(px(26.0))
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
        .hover(move |s| s.bg(pal_hover).text_color(pal_text_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(icon)
}

fn render_pill_action_btn<F: Fn(&mut App) + 'static>(
    label: &'static str,
    tooltip: &'static str,
    pal: &crate::ui::theme::ThemePalette,
    on_click: F,
) -> impl IntoElement {
    let _ = tooltip;
    let pal_hover = pal.card_hover;
    let pal_text_hover = pal.primary_accent;
    div()
        .px_2()
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
        .hover(move |s| s.bg(pal_hover).text_color(pal_text_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(label)
}
