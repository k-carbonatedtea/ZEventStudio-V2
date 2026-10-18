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
    render_diff_viewer_view, render_event_manager_view, render_json_viewer_view,
    render_welcome_view,
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

        // 默认极速性能模式调度：活跃窗口高刷新流畅渲染，或文本框处于激活状态（用于闪烁光标动画）
        let has_active_field = self.model.read(cx).has_active_text_input();
        if is_active || has_active_field {
            cx.on_next_frame(window, |_, _, cx| {
                cx.notify();
            });
        }

        let current_tab = self.model.read(cx).current_tab;

        // 焦点调度：
        // 1. 如果存在活跃的输入框（如全局搜索、Git提交、设置弹窗），确保应用主 FocusHandle 处于聚焦状态以接收输入
        // 2. 如果处于代码编辑视图且没有其他输入框活跃，确保代码编辑器实例获得焦点以支持文本输入、光标与 IME
        if has_active_field {
            if !self.focus_handle.is_focused(window) {
                self.focus_handle.focus(window, cx);
            }
        } else if current_tab == AppWorkspaceTab::CodeEditor {
            if let Some(ref editor) = self.code_editor {
                let editor_focused = editor.read(cx).focus_handle(cx).is_focused(window);
                if !editor_focused {
                    editor.update(cx, |ed, cx| ed.focus(window, cx));
                }
            }
        }

        let state = self.model.read(cx);
        if state.should_quit_app {
            cx.quit();
        }

        let current_fps = self.fps;
        let current_frame_time = self.frame_time_ms;

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
            // 挂载全局 IME 桥接 Element (在 paint 阶段注册 handle_input，激活 Windows 原生中文输入法)
            .child(crate::ui::components::ImeInputBridge::new(self.model.clone(), self.focus_handle.clone()))
            // 侧边栏与弹窗拖拽调宽全局鼠标移动监听
            .on_mouse_move(move |e, _window, cx| {
                let mx: f32 = e.position.x.into();
                let my: f32 = e.position.y.into();
                model_root_move.update(cx, |st, cx| {
                    if st.is_resizing_sidebar {
                        st.set_sidebar_width(mx);
                        cx.notify();
                    } else if let Some(((sx, sy), (w0, h0))) = st.settings_resize_start {
                        let delta_x = (mx - sx) * 2.0;
                        let delta_y = (my - sy) * 2.0;
                        st.settings_modal_width = (w0 + delta_x).clamp(680.0, 1600.0);
                        st.settings_modal_height = (h0 + delta_y).clamp(460.0, 1000.0);
                        st.settings_modal_maximized = false;
                        cx.notify();
                    }
                });
            })
            // 鼠标释放监听
            .on_mouse_up(MouseButton::Left, move |_e, _window, cx| {
                model_root_up.update(cx, |st, cx| {
                    let mut changed = false;
                    if st.is_resizing_sidebar {
                        st.is_resizing_sidebar = false;
                        changed = true;
                    }
                    if st.settings_resize_start.is_some() {
                        st.settings_resize_start = None;
                        st.is_resizing_settings = false;
                        changed = true;
                    }
                    if changed {
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

                let clipboard = if event.keystroke.modifiers.control && event.keystroke.key == "v" {
                    cx.read_from_clipboard().and_then(|item| item.text())
                } else {
                    None
                };

                model_key.update(cx, |st, cx| {
                    if event.keystroke.modifiers.control && event.keystroke.key == "c" {
                        if let Some(active_text) = st.active_text_input() {
                            cx.write_to_clipboard(ClipboardItem::new_string(active_text.to_string()));
                            cx.stop_propagation();
                            return;
                        }
                    }

                    let should_notify = handle_global_key_event(event, st, clipboard.as_deref());
                    if should_notify {
                        cx.stop_propagation();
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
                        AppWorkspaceTab::Welcome => render_welcome_view(state, &self.model).into_any_element(),
                        AppWorkspaceTab::Flowchart => wrap_sidebar_workspace(
                            state,
                            &self.model,
                            render_flow_canvas(state, &self.model),
                            Some(render_inspector(state, &self.model).into_any_element()),
                        )
                        .into_any_element(),
                        AppWorkspaceTab::CodeEditor => wrap_sidebar_workspace(
                            state,
                            &self.model,
                            render_code_editor_view(state, &self.model, self.code_editor.as_ref()),
                            None,
                        )
                        .into_any_element(),
                        AppWorkspaceTab::DiffViewer => wrap_sidebar_workspace(
                            state,
                            &self.model,
                            render_diff_viewer_view(state, &self.model),
                            None,
                        )
                        .into_any_element(),
                        AppWorkspaceTab::Actors => wrap_fullscreen_tab(state, &self.model, render_actor_manager_view(state, &self.model)).into_any_element(),
                        AppWorkspaceTab::Events => wrap_fullscreen_tab(state, &self.model, render_event_manager_view(state, &self.model)).into_any_element(),
                        AppWorkspaceTab::Json => wrap_fullscreen_tab(state, &self.model, render_json_viewer_view(state, &self.model)).into_any_element(),
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
            // 标签页右键上下文菜单
            .children(state.tab_context_menu.as_ref().map(|(tid, pos)| {
                crate::ui::render_tab_context_menu(state, tid, *pos, &self.model)
            }))
            // Git 提交记录右键上下文菜单
            .children(state.git_commit_context_menu.as_ref().map(|(hash, pos)| {
                crate::ui::sidebar::render_commit_context_menu(state, hash, *pos, &self.model)
            }))
            // 节点属性与分支编辑模态框
            .children((state.active_node_editor_modal.is_some()).then(|| {
                render_node_editor_modal(state, &self.model)
            }))
            // 未保存修改关闭确认模态框
            .children((state.unsaved_changes_modal.is_some()).then(|| {
                crate::ui::render_unsaved_changes_modal(state, &self.model)
            }))
            // 首选项与系统设置模态框
            .children(state.settings_modal_open.then(|| {
                crate::ui::render_settings_modal(state, &self.model)
            }))
            // Git 代码修改与差异对比模态框
            .children((state.active_diff_modal.is_some()).then(|| {
                crate::ui::sidebar::render_git_diff_modal(state, &self.model)
            }))
    }
}

/// 包装带侧边栏和顶部文档标签栏的主工作区布局
fn wrap_sidebar_workspace(
    state: &AppState,
    model: &Entity<AppState>,
    body: impl IntoElement,
    extra_right: Option<gpui::AnyElement>,
) -> impl IntoElement {
    div()
        .flex()
        .size_full()
        .overflow_hidden()
        .child(render_sidebar(state, model))
        .children(state.sidebar_open.then(|| crate::ui::sidebar::render_sidebar_resize_handle(state, model)))
        .child(
            div()
                .flex_1()
                .flex()
                .size_full()
                .overflow_hidden()
                .capture_any_mouse_down({
                    let model = model.clone();
                    move |_e, _window, cx| {
                        model.update(cx, |st, cx| {
                            if st.has_active_text_input() {
                                st.clear_active_text_inputs();
                                cx.notify();
                            }
                        });
                    }
                })
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .size_full()
                        .overflow_hidden()
                        .child(crate::ui::render_document_tab_bar(state, model))
                        .child(
                            div()
                                .flex_1()
                                .size_full()
                                .overflow_hidden()
                                .child(body),
                        )
                        .child(render_bottom_tabs(state, model)),
                )
                .children(extra_right),
        )
}

/// 包装全屏单视图工作区布局 (包含底部模式切换菜单栏)
fn wrap_fullscreen_tab(
    state: &AppState,
    model: &Entity<AppState>,
    body: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .size_full()
        .overflow_hidden()
        .child(
            div()
                .flex_1()
                .size_full()
                .overflow_hidden()
                .child(body),
        )
        .child(render_bottom_tabs(state, model))
}