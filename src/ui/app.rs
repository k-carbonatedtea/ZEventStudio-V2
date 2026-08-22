use gpui::*;
use std::time::Instant;
use crate::ui::canvas::{render_flow_canvas, render_node_editor_modal};
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

        // 观察 AppState 数据变化，响应式精准触发视图重绘
        cx.observe(&model, |_, _, cx| {
            cx.notify();
        })
        .detach();

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

        // 极速性能模式调度：仅当显式开启性能模式且窗口激活时，才在每一帧请求下一帧刷新
        if is_perf && is_active {
            cx.on_next_frame(window, |_, _, cx| {
                cx.notify();
            });
        }

        let state = self.model.read(cx);
        let current_fps = self.fps;
        let current_frame_time = self.frame_time_ms;
        let current_tab = state.current_tab;

        // 3秒自动隐藏 Toast 提示气泡并触发平滑重绘
        let visible_toast = if let Some(ref t) = state.toast {
            let elapsed = t.created_at.elapsed().as_secs_f32();
            if elapsed >= t.duration_secs {
                let model_dismiss = self.model.clone();
                cx.spawn(async move |_this, cx| {
                    cx.update(|cx| {
                        model_dismiss.update(cx, |st, cx| {
                            if let Some(ref cur) = st.toast {
                                if cur.created_at.elapsed().as_secs_f32() >= cur.duration_secs {
                                    st.toast = None;
                                    cx.notify();
                                }
                            }
                        });
                    });
                }).detach();
                None
            } else {
                let remaining_ms = ((t.duration_secs - elapsed) * 1000.0).max(50.0) as u64;
                let model_timer = self.model.clone();
                cx.spawn(async move |_this, cx| {
                    smol::Timer::after(std::time::Duration::from_millis(remaining_ms + 10)).await;
                    cx.update(|cx| {
                        model_timer.update(cx, |st, cx| {
                            if let Some(ref cur) = st.toast {
                                if cur.created_at.elapsed().as_secs_f32() >= cur.duration_secs {
                                    st.toast = None;
                                    cx.notify();
                                }
                            }
                        });
                    });
                }).detach();
                Some(t.clone())
            }
        } else {
            None
        };



        let model_key = self.model.clone();
        let model_root_move = self.model.clone();
        let model_root_up = self.model.clone();

        div()
            .id("app_root_view")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .bg(hsla(240.0 / 360.0, 0.06, 0.07, 1.0))
            .font_family(".SystemUIFont")
            .text_color(rgb(0xffffff))
            // 侧边栏拖拽调宽全局鼠标移动监听
            .on_mouse_move(move |e, _window, cx| {
                let mx: f32 = e.position.x.into();
                model_root_move.update(cx, |st, cx| {
                    if st.is_resizing_sidebar {
                        st.set_sidebar_width(mx);
                        cx.notify();
                    }
                });
            })
            // 侧边栏拖拽调宽鼠标释放
            .on_mouse_up(MouseButton::Left, move |_e, _window, cx| {
                model_root_up.update(cx, |st, cx| {
                    if st.is_resizing_sidebar {
                        st.is_resizing_sidebar = false;
                        cx.notify();
                    }
                });
            })
            // 全局键盘输入拦截器：快捷键与实体键盘键入
            .on_key_down(move |event, _window, cx| {
                if event.keystroke.modifiers.control && event.keystroke.key == "o" {
                    crate::ui::views::welcome::quick_tab::pick_and_load_file(&model_key, cx);
                    return;
                }
                if event.keystroke.modifiers.control && event.keystroke.key == "n" {
                    model_key.update(cx, |st, cx| {
                        st.new_blank_flow();
                        cx.notify();
                    });
                    return;
                }

                let mut modal_handled = false;
                model_key.update(cx, |st, cx| {
                    if let Some(ref mut modal) = st.active_node_editor_modal {
                        modal_handled = true;
                        let mut should_notify = false;
                        match modal.active_field {
                            crate::ui::canvas::node_editor::ActiveEditField::FlowchartName => {
                                let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.res_flowchart_name, event);
                                if is_esc { modal.active_field = crate::ui::canvas::node_editor::ActiveEditField::None; }
                                if is_enter { modal.active_field = crate::ui::canvas::node_editor::ActiveEditField::EntryPointName; }
                                should_notify = changed || is_enter || is_esc;
                            }
                            crate::ui::canvas::node_editor::ActiveEditField::EntryPointName => {
                                let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.entry_point_name, event);
                                if is_esc || is_enter { modal.active_field = crate::ui::canvas::node_editor::ActiveEditField::None; }
                                should_notify = changed || is_enter || is_esc;
                            }
                            crate::ui::canvas::node_editor::ActiveEditField::BranchValue(idx) => {
                                if idx < modal.branches.len() {
                                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.branches[idx].value, event);
                                    if is_esc || is_enter { modal.active_field = crate::ui::canvas::node_editor::ActiveEditField::None; }
                                    should_notify = changed || is_enter || is_esc;
                                }
                            }
                            crate::ui::canvas::node_editor::ActiveEditField::BranchTarget(idx) => {
                                if idx < modal.branches.len() {
                                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.branches[idx].target_id, event);
                                    if is_esc || is_enter { modal.active_field = crate::ui::canvas::node_editor::ActiveEditField::None; }
                                    should_notify = changed || is_enter || is_esc;
                                }
                            }
                            crate::ui::canvas::node_editor::ActiveEditField::ParamKey(idx) => {
                                if idx < modal.properties.len() {
                                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.properties[idx].key, event);
                                    if is_esc { modal.active_field = crate::ui::canvas::node_editor::ActiveEditField::None; }
                                    if is_enter { modal.active_field = crate::ui::canvas::node_editor::ActiveEditField::ParamVal(idx); }
                                    should_notify = changed || is_enter || is_esc;
                                }
                            }
                            crate::ui::canvas::node_editor::ActiveEditField::ParamVal(idx) => {
                                if idx < modal.properties.len() {
                                    let (changed, is_enter, is_esc) = handle_text_key_event(&mut modal.properties[idx].val, event);
                                    if is_esc || is_enter { modal.active_field = crate::ui::canvas::node_editor::ActiveEditField::None; }
                                    should_notify = changed || is_enter || is_esc;
                                }
                            }
                            crate::ui::canvas::node_editor::ActiveEditField::None => {
                                if event.keystroke.key == "escape" {
                                    st.active_node_editor_modal = None;
                                    should_notify = true;
                                } else if event.keystroke.key == "enter" {
                                    st.save_node_editor();
                                    should_notify = true;
                                }
                            }
                        }
                        if should_notify {
                            cx.notify();
                        }
                    }
                });

                if modal_handled {
                    return;
                }



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
            // 全局主工作区主体 (根据底部 Tab 切换：启动页 / 流程图 / 角色管理 / 事件列表 / JSON 源码)
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .overflow_hidden()
                    .child(match current_tab {
                        AppWorkspaceTab::Welcome => {
                            crate::ui::views::render_welcome_view(state, &self.model).into_any_element()
                        }
                        AppWorkspaceTab::Flowchart => div()
                            .flex()
                            .size_full()
                            .overflow_hidden()
                            .child(render_sidebar(state, &self.model))
                            .child(crate::ui::sidebar::render_sidebar_resize_handle(state, &self.model))
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

            // 底部工作区选项卡栏 (仅在节点布局图/编辑器等视图中显示)
            .children((current_tab != AppWorkspaceTab::Welcome).then(|| {
                render_bottom_tabs(state, &self.model)
            }))
            // Floating Toast notification (自动3秒后消失)
            .children(visible_toast.map(|t| {
                div()
                    .absolute()
                    .bottom_12()
                    .right_4()
                    .child(render_toast(&t))
            }))
            // 节点属性与分支编辑模态框 (Node Editor Modal)
            .children((state.active_node_editor_modal.is_some()).then(|| {
                render_node_editor_modal(state, &self.model)
            }))


    }
}

