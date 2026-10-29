use gpui::*;
use gpui_component::input::EditorState;
use std::time::Instant;
use crate::ui::app_layout::{wrap_fullscreen_tab, wrap_sidebar_workspace};
use crate::ui::canvas::{render_flow_canvas, render_node_editor_modal};
use crate::ui::components::render_toast;
use crate::ui::inspector::render_inspector;
use crate::ui::key_handler::handle_global_key_event;
use crate::ui::render_title_bar;
use crate::ui::state::{AppState, AppWorkspaceTab};
use crate::ui::toolbar::render_toolbar;
use crate::ui::views::{
    render_actor_manager_view, render_code_editor_view,
    render_diff_viewer_view, render_event_manager_view,
    render_quest_map_view, render_timeline_editor_view, render_welcome_view,
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
    pub code_editor_version: usize,
    pub msbt_bubble_editor: Option<Entity<EditorState>>,
    pub msbt_bubble_tab_id: Option<String>,
    pub msbt_bubble_entry_key: Option<String>,
    pub msbt_bubble_last_draft: String,
    pub was_window_active: bool,
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
            code_editor_version: 0,
            msbt_bubble_editor: None,
            msbt_bubble_tab_id: None,
            msbt_bubble_entry_key: None,
            msbt_bubble_last_draft: String::new(),
            was_window_active: false,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_active = window.is_window_active();

        // 窗口从后台切回前台时，自动检测 Git 状态与活动文件磁盘外部变更
        if is_active && !self.was_window_active {
            self.model.update(cx, |st, cx| {
                st.refresh_git_status();
                st.check_and_reload_active_tab_if_disk_changed();
                cx.notify();
            });
        }
        self.was_window_active = is_active;

        // 1. 同步与管理高性能可编辑代码编辑器实例 (EditorState) 与 MSBT 气泡对话编辑器实例
        self.sync_code_editor(window, cx);
        self.sync_msbt_bubble_editor(window, cx);

        // 采样并计算真实渲染帧率
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();

        if elapsed >= 0.25 {
            let measured = (self.frame_count as f32 / elapsed).round().max(1.0);
            // 平滑移动平均，避免 250ms 采样时间窗微小偏差导致的数值抖动
            self.fps = if self.fps <= 0.0 { measured } else { (self.fps * 0.35 + measured * 0.65).round() };
            self.frame_time_ms = if self.fps > 0.0 { 1000.0 / self.fps } else { 0.0 };
            self.frame_count = 0;
            self.last_fps_update = now;
        }

        // 推进时间线回放游标
        let is_timeline_playing = self.model.read(cx).active_timeline_state().is_some_and(|ts| ts.is_playing);
        if is_timeline_playing {
            self.model.update(cx, |st, cx| {
                if let Some(ts) = st.active_timeline_state_mut() {
                    if ts.is_playing {
                        ts.advance_frame(elapsed);
                        cx.notify();
                    }
                }
            });
        }

        // 默认极速性能模式调度：活跃窗口高刷新流畅渲染，或文本框激活（光标动画），或正在提取地图标点（旋转动画），或时间线回放
        let has_active_field = self.model.read(cx).has_active_text_input();
        let is_extracting_markers = self.model.read(cx).quest_map_is_extracting_markers;
        if is_active || has_active_field || is_extracting_markers || is_timeline_playing {
            cx.on_next_frame(window, |_, _, cx| {
                cx.notify();
            });
        }

        let current_tab = self.model.read(cx).current_tab;
        let needs_markers = current_tab == AppWorkspaceTab::Quests
            && self.model.read(cx).quest_map_markers_cache.is_none()
            && !self.model.read(cx).quest_map_is_extracting_markers;
        if needs_markers {
            let model = self.model.clone();
            self.model.update(cx, |st, cx| {
                st.trigger_async_extract_hyrule_markers(&model, cx);
            });
        }

        // 焦点调度：
        // 1. 如果存在活跃的输入框（如全局搜索、Git提交、设置弹窗），确保应用主 FocusHandle 处于聚焦状态以接收输入
        // 2. 如果处于代码编辑视图且没有其他输入框活跃，确保代码编辑器实例获得焦点以支持文本输入、光标与 IME
        let is_bubble_active = {
            let state = self.model.read(cx);
            let active_id = state.active_tab_id.as_deref().unwrap_or("");
            state.open_tabs.iter().find(|t| t.id == active_id).is_some_and(|tab| {
                tab.code_state.as_ref().is_some_and(|c| {
                    c.format == crate::model::code_document::CodeFormat::Msbt
                        && c.msbt_view_mode == crate::model::code_document::MsbtViewMode::Bubble
                })
            })
        };

        if has_active_field {
            if !self.focus_handle.is_focused(window) {
                self.focus_handle.focus(window, cx);
            }
        } else if current_tab == AppWorkspaceTab::CodeEditor {
            if is_bubble_active {
                if let Some(ref ed) = self.msbt_bubble_editor {
                    let editor_focused = ed.read(cx).focus_handle(cx).is_focused(window);
                    if !editor_focused {
                        ed.update(cx, |editor, cx| editor.focus(window, cx));
                    }
                }
            } else if let Some(ref editor) = self.code_editor {
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
                    if st.text_input_is_dragging {
                        st.text_input_is_dragging = false;
                        if st.text_input_select_anchor == Some(st.text_input_cursor) {
                            st.text_input_select_anchor = None;
                        }
                        changed = true;
                    }
                    if changed {
                        cx.notify();
                    }
                });
            })
            // 全局键盘输入拦截器：快捷键与实体键盘键入
            .on_key_down(move |event, _window, cx| {
                if event.keystroke.modifiers.control && event.keystroke.key.eq_ignore_ascii_case("o") {
                    crate::ui::views::welcome::quick_tab::pick_and_load_file(&model_key, cx);
                    return;
                }

                if event.keystroke.key == "space"
                    && !model_key.read(cx).has_active_text_input()
                    && model_key.read(cx).current_tab == AppWorkspaceTab::Timeline
                {
                    model_key.update(cx, |st, cx| {
                        if let Some(ts) = st.active_timeline_state_mut() {
                            ts.toggle_play();
                            cx.notify();
                        }
                    });
                    return;
                }

                let clipboard = if event.keystroke.modifiers.control && event.keystroke.key.eq_ignore_ascii_case("v") {
                    cx.read_from_clipboard().and_then(|item| item.text())
                } else {
                    None
                };

                model_key.update(cx, |st, cx| {
                    if crate::ui::key_handler::handle_clipboard_key_event(event, st, cx) {
                        cx.stop_propagation();
                        return;
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
                        AppWorkspaceTab::Flowchart => {
                            if state.open_tabs.is_empty() {
                                render_welcome_view(state, &self.model).into_any_element()
                            } else {
                                wrap_sidebar_workspace(
                                    state,
                                    &self.model,
                                    render_flow_canvas(state, &self.model),
                                    Some(render_inspector(state, &self.model).into_any_element()),
                                )
                                .into_any_element()
                            }
                        }
                        AppWorkspaceTab::Timeline => {
                            if state.open_tabs.is_empty() {
                                render_welcome_view(state, &self.model).into_any_element()
                            } else {
                                wrap_sidebar_workspace(
                                    state,
                                    &self.model,
                                    render_timeline_editor_view(state, &self.model),
                                    None,
                                )
                                .into_any_element()
                            }
                        }
                        AppWorkspaceTab::CodeEditor => {
                            if state.open_tabs.is_empty() {
                                render_welcome_view(state, &self.model).into_any_element()
                            } else {
                                wrap_sidebar_workspace(
                                    state,
                                    &self.model,
                                    render_code_editor_view(
                                        state,
                                        &self.model,
                                        self.code_editor.as_ref(),
                                        self.msbt_bubble_editor.as_ref(),
                                    ),
                                    None,
                                )
                                .into_any_element()
                            }
                        }
                        AppWorkspaceTab::DiffViewer => wrap_sidebar_workspace(
                            state,
                            &self.model,
                            render_diff_viewer_view(state, &self.model),
                            None,
                        )
                        .into_any_element(),
                        AppWorkspaceTab::Actors => wrap_fullscreen_tab(state, &self.model, render_actor_manager_view(state, &self.model)).into_any_element(),
                        AppWorkspaceTab::Events => wrap_fullscreen_tab(state, &self.model, render_event_manager_view(state, &self.model)).into_any_element(),
                        AppWorkspaceTab::Quests => wrap_sidebar_workspace(
                            state,
                            &self.model,
                            render_quest_map_view(state, &self.model),
                            None,
                        )
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
            // 标签页右键上下文菜单
            .children(state.tab_context_menu.as_ref().map(|(tid, pos)| {
                crate::ui::render_tab_context_menu(state, tid, *pos, &self.model)
            }))
            // Git 提交记录右键上下文菜单
            .children(state.file_tree_context_menu.as_ref().map(|menu| {
                crate::ui::sidebar::render_file_tree_context_menu(state, menu, &self.model)
            }))
            .children(state.rename_modal.as_ref().map(|modal| {
                crate::ui::sidebar::render_rename_modal(state, modal, &self.model)
            }))
            .children(state.delete_confirm_modal.as_ref().map(|modal| {
                crate::ui::sidebar::render_delete_confirm_modal(state, modal, &self.model)
            }))
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
            // 新建与编辑任务向导模态框
            .children(state.quest_creator_modal.as_ref().map(|modal| {
                crate::ui::sidebar::render_quest_creator_modal(modal, state, &self.model)
            }))
            // 任务原始数据与拓扑查看模态框
            .children(state.quest_raw_data_modal.as_ref().map(|modal| {
                crate::ui::sidebar::render_quest_raw_data_modal(modal, state, &self.model)
            }))
    }
}

