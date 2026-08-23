use gpui::*;
use gpui_component::input::{EditorState, InputEvent};
use std::time::Instant;
use crate::ui::canvas::{render_flow_canvas, render_node_editor_modal};
use crate::ui::components::render_toast;
use crate::ui::inspector::render_inspector;
use crate::ui::key_handler::handle_global_key_event;
use crate::ui::render_title_bar;
use crate::ui::sidebar::render_sidebar;
use crate::ui::state::{AppState, AppWorkspaceTab};
use crate::ui::toolbar::render_toolbar;
use crate::ui::views::{
    render_actor_manager_view, render_bottom_tabs, render_code_editor_view,
    render_event_manager_view, render_json_viewer_view, render_welcome_view,
};

pub struct AppView {
    pub model: Entity<AppState>,
    pub frame_count: u32,
    pub last_fps_update: Instant,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub focus_handle: FocusHandle,
    pub code_editor: Option<Entity<EditorState>>,
    pub code_editor_tab_id: Option<String>,
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
            code_editor: None,
            code_editor_tab_id: None,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_active = window.is_window_active();

        // 1. 同步与管理高性能可编辑代码编辑器实例 (EditorState)
        let active_tab_info = {
            let state = self.model.read(cx);
            let active_id = state.active_tab_id.clone();
            active_id.and_then(|id| {
                state.open_tabs.iter().find(|t| t.id == id).and_then(|tab| {
                    tab.code_state.as_ref().map(|code| {
                        (id, code.content.clone(), code.format.language_id())
                    })
                })
            })
        };

        if let Some((current_id, code_content, lang)) = active_tab_info {
            if self.code_editor_tab_id.as_ref() != Some(&current_id) {
                let new_editor = cx.new(|cx| {
                    EditorState::new(window, cx)
                        .language(lang)
                        .line_number(true)
                        .default_value(code_content)
                });

                let model_sub = self.model.clone();
                let target_tab_id = current_id.clone();
                cx.subscribe_in(
                    &new_editor,
                    window,
                    move |_this: &mut AppView, editor, event: &InputEvent, _window, cx| {
                        if let InputEvent::Change = event {
                            let new_val = editor.read(cx).value();
                            let tid = target_tab_id.clone();
                            model_sub.update(cx, |st, cx| {
                                if let Some(tab) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut code) = tab.code_state {
                                        code.content = new_val.to_string();
                                    }
                                    tab.recompute_modified();
                                    cx.notify();
                                }
                            });
                        }
                    },
                )
                .detach();

                self.code_editor = Some(new_editor);
                self.code_editor_tab_id = Some(current_id);
            }
        }

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

        // 极速性能模式调度
        if is_perf && is_active {
            cx.on_next_frame(window, |_, _, cx| {
                cx.notify();
            });
        }

        let state = self.model.read(cx);
        let current_fps = self.fps;
        let current_frame_time = self.frame_time_ms;
        let current_tab = state.current_tab;

        // 3秒自动隐藏 Toast 提示气泡 (过滤已过期 Toast)
        let visible_toast = state
            .toast
            .as_ref()
            .filter(|t| t.created_at.elapsed().as_secs_f32() < t.duration_secs);

        let model_key = self.model.clone();
        let model_root_move = self.model.clone();
        let model_root_up = self.model.clone();

        div()
            .id("app_root_view")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .bg(state.theme().app_bg)
            .font_family(".SystemUIFont")
            .text_color(state.theme().text_primary)
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

                model_key.update(cx, |st, cx| {
                    let should_notify = handle_global_key_event(event, st);
                    if should_notify {
                        cx.notify();
                    }
                });
            })
            // 顶部沉浸式自定义标题栏与菜单栏 (TitleBar & Menu Bar)
            .child(render_title_bar(state, &self.model))
            // 顶部工具栏 (Toolbar)
            .child(render_toolbar(state, current_fps, current_frame_time, &self.model))
            // 全局主工作区主体 (根据工作区状态切换)
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .overflow_hidden()
                    .child(match current_tab {
                        AppWorkspaceTab::Welcome => {
                            render_welcome_view(state, &self.model).into_any_element()
                        }
                        AppWorkspaceTab::Flowchart => div()
                            .flex()
                            .size_full()
                            .overflow_hidden()
                            .child(render_sidebar(state, &self.model))
                            .child(crate::ui::sidebar::render_sidebar_resize_handle(state, &self.model))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .size_full()
                                    .overflow_hidden()
                                    // 中间主工作区列 (顶部文档Tab + 流程图画布 + 底部切换栏)
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .size_full()
                                            .overflow_hidden()
                                            // 顶部多文档/多文件标签栏 (Document Tab Bar)
                                            .child(crate::ui::render_document_tab_bar(state, &self.model))
                                            // 流程图画布主体
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .size_full()
                                                    .overflow_hidden()
                                                    .child(render_flow_canvas(state, &self.model)),
                                            )
                                            // 流程图窗口底部的模式切换菜单栏
                                            .child(render_bottom_tabs(state, &self.model)),
                                    )
                                    // 右侧属性与拓扑检查器
                                    .child(render_inspector(state, &self.model)),
                            )
                            .into_any_element(),

                        AppWorkspaceTab::CodeEditor => div()
                            .flex()
                            .size_full()
                            .overflow_hidden()
                            .child(render_sidebar(state, &self.model))
                            .child(crate::ui::sidebar::render_sidebar_resize_handle(state, &self.model))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .size_full()
                                    .overflow_hidden()
                                    // 顶部多文档/多文件标签栏 (Document Tab Bar)
                                    .child(crate::ui::render_document_tab_bar(state, &self.model))
                                    // 代码编辑器主体
                                    .child(
                                        div()
                                            .flex_1()
                                            .size_full()
                                            .overflow_hidden()
                                            .child(render_code_editor_view(state, &self.model, self.code_editor.as_ref())),
                                    )
                                    // 代码编辑器窗口底部的模式切换菜单栏
                                    .child(render_bottom_tabs(state, &self.model)),
                            )
                            .into_any_element(),

                        AppWorkspaceTab::Actors => div()
                            .flex()
                            .flex_col()
                            .size_full()
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex_1()
                                    .size_full()
                                    .overflow_hidden()
                                    .child(render_actor_manager_view(state, &self.model)),
                            )
                            .child(render_bottom_tabs(state, &self.model))
                            .into_any_element(),

                        AppWorkspaceTab::Events => div()
                            .flex()
                            .flex_col()
                            .size_full()
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex_1()
                                    .size_full()
                                    .overflow_hidden()
                                    .child(render_event_manager_view(state, &self.model)),
                            )
                            .child(render_bottom_tabs(state, &self.model))
                            .into_any_element(),

                        AppWorkspaceTab::Json => div()
                            .flex()
                            .flex_col()
                            .size_full()
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex_1()
                                    .size_full()
                                    .overflow_hidden()
                                    .child(render_json_viewer_view(state, &self.model)),
                            )
                            .child(render_bottom_tabs(state, &self.model))
                            .into_any_element(),
                    }),
            )
            // Floating Toast notification
            .children(visible_toast.map(|t| {
                div()
                    .absolute()
                    .bottom_12()
                    .right_4()
                    .child(render_toast(t))
            }))
            // 节点属性与分支编辑模态框
            .children((state.active_node_editor_modal.is_some()).then(|| {
                render_node_editor_modal(state, &self.model)
            }))
            // 未保存修改关闭确认模态框
            .children((state.unsaved_changes_modal.is_some()).then(|| {
                crate::ui::render_unsaved_changes_modal(state, &self.model)
            }))
    }
}
