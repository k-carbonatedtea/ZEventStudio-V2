use gpui::*;
use std::time::{Duration, Instant};
use crate::ui::canvas::render_flow_canvas;
use crate::ui::components::{handle_text_key_event, render_toast};
use crate::ui::inspector::render_inspector;
use crate::ui::sidebar::render_sidebar;
use crate::ui::state::{AppState, AppWorkspaceTab};
use crate::ui::toolbar::render_toolbar;
use crate::ui::views::{
    render_actor_manager_view, render_bottom_tabs, render_event_manager_view, render_json_viewer_view,
};

pub struct AppView {
    pub model: Entity<AppState>,
    pub frame_count: u32,
    pub last_fps_update: Instant,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub focus_handle: FocusHandle,
}

impl AppView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let model = cx.new(|_| AppState::new());
        let focus_handle = cx.focus_handle();
        Self {
            model,
            frame_count: 0,
            last_fps_update: Instant::now(),
            fps: 60.0,
            frame_time_ms: 1.0,
            focus_handle,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_active = window.is_window_active();

        let state = self.model.read(cx);
        let is_perf = state.performance_mode;

        // 采样并计算真实渲染帧率
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();

        if elapsed >= 0.25 {
            self.fps = ((self.frame_count as f32 / elapsed).round()).max(1.0);
            self.frame_time_ms = if self.fps > 0.0 { 1000.0 / self.fps } else { 0.0 };
            self.frame_count = 0;
            self.last_fps_update = now;
        }

        // 动态帧率控制与性能模式调度
        if is_perf && is_active {
            cx.on_next_frame(window, |_, _, cx| {
                cx.notify();
            });
            cx.spawn_in(window, async move |view, cx| {
                let _ = cx.update(|_window, cx| {
                    let _ = view.update(cx, |_, cx| {
                        cx.notify();
                    });
                });
            })
            .detach();
        } else {
            let sleep_ms = if is_active { 7 } else { 33 };
            cx.spawn_in(window, async move |view, cx| {
                smol::Timer::after(Duration::from_millis(sleep_ms)).await;
                let _ = cx.update(|_window, cx| {
                    let _ = view.update(cx, |_, cx| {
                        cx.notify();
                    });
                });
            })
            .detach();
        }

        let state = self.model.read(cx);
        let toast = state.toast.clone();
        let current_fps = self.fps;
        let current_frame_time = self.frame_time_ms;
        let current_tab = state.current_tab;

        let model_key = self.model.clone();

        div()
            .id("app_root_view")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .bg(hsla(220.0 / 360.0, 0.35, 0.08, 1.0))
            .font_family(".SystemUIFont")
            .text_color(rgb(0xffffff))
            // 全局键盘输入拦截器：当正在添加动作/查询/参数/角色时，直接捕获物理键盘键入
            .on_key_down(move |event, _window, cx| {
                model_key.update(cx, |st, cx| {
                    if st.is_adding_action {
                        let (changed, is_enter, is_esc) = handle_text_key_event(&mut st.new_action_name, event);
                        if is_enter {
                            let act_name = if st.new_action_name.trim().is_empty() {
                                "Demo_WaitFrame".to_string()
                            } else {
                                st.new_action_name.trim().to_string()
                            };
                            if let Some(act) = st.graph.actors.get_mut(st.selected_actor_idx) {
                                if !act.actions.contains(&act_name) {
                                    act.actions.push(act_name);
                                }
                            }
                            st.is_adding_action = false;
                        } else if is_esc {
                            st.is_adding_action = false;
                        }
                        if changed || is_enter || is_esc {
                            cx.notify();
                        }
                    } else if st.is_adding_query {
                        let (changed, is_enter, is_esc) = handle_text_key_event(&mut st.new_query_name, event);
                        if is_enter {
                            let qry_name = if st.new_query_name.trim().is_empty() {
                                "CheckFlag".to_string()
                            } else {
                                st.new_query_name.trim().to_string()
                            };
                            if let Some(act) = st.graph.actors.get_mut(st.selected_actor_idx) {
                                if !act.queries.contains(&qry_name) {
                                    act.queries.push(qry_name);
                                }
                            }
                            st.is_adding_query = false;
                        } else if is_esc {
                            st.is_adding_query = false;
                        }
                        if changed || is_enter || is_esc {
                            cx.notify();
                        }
                    } else if st.is_adding_actor {
                        let (changed, is_enter, is_esc) = handle_text_key_event(&mut st.new_actor_name, event);
                        if is_enter {
                            let name = if st.new_actor_name.trim().is_empty() {
                                format!("Actor_{}", st.graph.actors.len())
                            } else {
                                st.new_actor_name.trim().to_string()
                            };
                            let mut new_a = crate::model::actor::ActorData::new(name);
                            new_a.load_default_params();
                            st.graph.actors.push(new_a);
                            st.selected_actor_idx = st.graph.actors.len() - 1;
                            st.is_adding_actor = false;
                        } else if is_esc {
                            st.is_adding_actor = false;
                        }
                        if changed || is_enter || is_esc {
                            cx.notify();
                        }
                    } else if st.is_adding_param {
                        let (changed, is_enter, is_esc) = handle_text_key_event(&mut st.new_param_key, event);
                        if is_enter {
                            if let Some(act) = st.graph.actors.get_mut(st.selected_actor_idx) {
                                act.params.push(crate::model::node::PropertyItem {
                                    key: if st.new_param_key.trim().is_empty() { format!("Param_{}", act.params.len()) } else { st.new_param_key.trim().to_string() },
                                    val: if st.new_param_val.trim().is_empty() { "0".to_string() } else { st.new_param_val.trim().to_string() },
                                    val_type: st.new_param_type.clone(),
                                });
                            }
                            st.is_adding_param = false;
                        } else if is_esc {
                            st.is_adding_param = false;
                        }
                        if changed || is_enter || is_esc {
                            cx.notify();
                        }
                    }
                });
            })
            // 顶部工具栏 (Toolbar)
            .child(render_toolbar(state, current_fps, current_frame_time, &self.model))
            // 全局主工作区主体 (根据底部 Tab 切换：流程图 / 角色管理 / 事件列表 / JSON 源码)
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .overflow_hidden()
                    .child(match current_tab {
                        AppWorkspaceTab::Flowchart => div()
                            .flex()
                            .size_full()
                            .overflow_hidden()
                            .child(render_sidebar(state, &self.model))
                            .child(render_flow_canvas(state, &self.model))
                            .child(render_inspector(state, &self.model))
                            .into_any_element(),

                        AppWorkspaceTab::Actors => {
                            render_actor_manager_view(state, &self.model).into_any_element()
                        }
                        AppWorkspaceTab::Events => {
                            render_event_manager_view(state, &self.model).into_any_element()
                        }
                        AppWorkspaceTab::Json => {
                            render_json_viewer_view(state, &self.model).into_any_element()
                        }
                    }),
            )
            // 底部工作区全局选项卡栏 (Bottom Tabs Bar)
            .child(render_bottom_tabs(state, &self.model))
            // Floating Toast notification
            .children(toast.map(|t| {
                div()
                    .absolute()
                    .bottom_12()
                    .right_4()
                    .child(render_toast(&t))
            }))
    }
}
