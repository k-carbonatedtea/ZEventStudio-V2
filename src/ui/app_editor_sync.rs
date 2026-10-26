use gpui::*;
use gpui_component::input::{EditorState, InputEvent};
use crate::ui::app::AppView;

/// 为 AppView 提供代码编辑器 (MSYT 源码) 与气泡对话编辑器 (MSBT 条目) 的生命周期同步调度
impl AppView {
    /// 同步全屏代码编辑器实例 (支持 MSYT, BYML, AAMP, BFEVFL, JSON 等任意文本格式)
    pub fn sync_code_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active_tab_info = {
            let state = self.model.read(cx);
            let active_id = state.active_tab_id.clone();
            active_id.and_then(|id| {
                state.open_tabs.iter().find(|t| t.id == id).and_then(|tab| {
                    tab.code_state.as_ref().map(|code| {
                        (id, code.content.clone(), code.format.language_id(), code.version)
                    })
                })
            })
        };

        if let Some((current_id, code_content, lang, current_version)) = active_tab_info {
            if self.code_editor.is_none() || self.code_editor_tab_id.as_ref() != Some(&current_id) {
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
                self.code_editor_version = current_version;
            } else if self.code_editor_version != current_version {
                if let Some(ref ed) = self.code_editor {
                    ed.update(cx, |ed_state, cx| {
                        ed_state.set_value(code_content, window, cx);
                    });
                }
                self.code_editor_version = current_version;
            }
        } else if self.code_editor.is_some() {
            self.code_editor = None;
            self.code_editor_tab_id = None;
        }
    }

    /// 同步 MSBT 气泡对话编辑器条目实例 (提供与 MSYT 源码编辑器完全一致的光标、选区与快捷键体验)
    pub fn sync_msbt_bubble_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let needs_init = {
            let state = self.model.read(cx);
            state.active_tab_id.as_deref().and_then(|id| {
                state.open_tabs.iter().find(|t| t.id == id).and_then(|tab| {
                    tab.code_state.as_ref().map(|code| {
                        code.format == crate::model::code_document::CodeFormat::Msbt
                            && code.msbt_view_mode == crate::model::code_document::MsbtViewMode::Bubble
                            && state.msbt_selected_entry_key.is_none()
                    })
                })
            }).unwrap_or(false)
        };
        if needs_init {
            self.model.update(cx, |st, _| {
                st.sync_msbt_selected_entry_to_draft();
            });
        }

        let bubble_info = {
            let state = self.model.read(cx);
            let active_id = state.active_tab_id.clone();
            active_id.and_then(|id| {
                state.open_tabs.iter().find(|t| t.id == id).and_then(|tab| {
                    tab.code_state.as_ref().and_then(|code| {
                        if code.format == crate::model::code_document::CodeFormat::Msbt
                            && code.msbt_view_mode == crate::model::code_document::MsbtViewMode::Bubble
                        {
                            let entry_key = state.msbt_selected_entry_key.clone();
                            let draft = state.msbt_text_editor_draft.clone();
                            Some((id, entry_key, draft))
                        } else {
                            None
                        }
                    })
                })
            })
        };

        if let Some((tab_id, entry_key, draft)) = bubble_info {
            let needs_recreate = self.msbt_bubble_editor.is_none()
                || self.msbt_bubble_tab_id.as_ref() != Some(&tab_id)
                || self.msbt_bubble_entry_key != entry_key;

            if needs_recreate {
                let new_editor = cx.new(|cx| {
                    EditorState::new(window, cx)
                        .line_number(false)
                        .default_value(draft.clone())
                });

                let model_sub = self.model.clone();
                cx.subscribe_in(
                    &new_editor,
                    window,
                    move |this: &mut AppView, editor, event: &InputEvent, _window, cx| {
                        if let InputEvent::Change = event {
                            let new_val = editor.read(cx).value().to_string();
                            this.msbt_bubble_last_draft = new_val.clone();
                            model_sub.update(cx, |st, cx| {
                                st.msbt_text_editor_draft = new_val;
                                st.commit_msbt_draft_to_active_tab();
                                cx.notify();
                            });
                        }
                    },
                )
                .detach();

                self.msbt_bubble_editor = Some(new_editor);
                self.msbt_bubble_tab_id = Some(tab_id);
                self.msbt_bubble_entry_key = entry_key;
                self.msbt_bubble_last_draft = draft;
            } else if self.msbt_bubble_last_draft != draft {
                if let Some(ref ed) = self.msbt_bubble_editor {
                    let cur_ed_val = ed.read(cx).value();
                    if cur_ed_val.as_ref() != draft {
                        ed.update(cx, |ed_state, cx| {
                            ed_state.set_value(draft.clone(), window, cx);
                        });
                    }
                }
                self.msbt_bubble_last_draft = draft;
            }
        } else if self.msbt_bubble_editor.is_some() {
            self.msbt_bubble_editor = None;
            self.msbt_bubble_tab_id = None;
            self.msbt_bubble_entry_key = None;
            self.msbt_bubble_last_draft.clear();
        }
    }
}
