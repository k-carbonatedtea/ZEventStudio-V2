use crate::{
    model::map_editor::MapDocumentState,
    ui::{components::Toast, state::AppState},
};
use gpui::*;

impl AppState {
    pub fn active_map(&self) -> Option<&MapDocumentState> {
        self.open_tabs
            .iter()
            .find(|t| Some(t.id.as_str()) == self.active_tab_id.as_deref())?
            .map_state
            .as_ref()
    }
    pub fn active_map_mut(&mut self) -> Option<&mut MapDocumentState> {
        self.open_tabs
            .iter_mut()
            .find(|t| Some(t.id.as_str()) == self.active_tab_id.as_deref())?
            .map_state
            .as_mut()
    }
    pub fn start_pending_map_load(&mut self, cx: &mut Context<Self>) {
        let active_id = self.active_tab_id.clone();
        let Some(tab) = self.open_tabs.iter_mut().find(|t| {
            Some(&t.id) == active_id.as_ref()
                && t.map_state
                    .as_ref()
                    .is_some_and(|m| m.loading && !m.load_started)
        }) else {
            return;
        };
        let id = tab.id.clone();
        let map = tab.map_state.as_mut().unwrap();
        map.load_started = true;
        let snapshot = map.clone();
        let expected_root = snapshot.file.root.clone();
        let settings = self.game_paths.clone();
        let project = self.project_path.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        crate::model::map_editor::loader::load_scene(snapshot, settings, project)
                    }))
                    .unwrap_or_else(|_| Err("资源解析异常，请检查模型或地图资源是否完整".into()))
                })
                .await;
            let _ = this.update(cx, |st, cx| {
                if let Some(tab) = st.open_tabs.iter_mut().find(|t| t.id == id) {
                    if tab.map_state.as_ref().is_some_and(|m| {
                        m.loading
                            && m.load_started
                            && std::sync::Arc::ptr_eq(&m.file.root, &expected_root)
                    }) {
                        match result {
                            Ok(doc) => {
                                tab.map_state = Some(doc);
                                tab.recompute_modified();
                            }
                            Err(e) => {
                                if let Some(map) = &mut tab.map_state {
                                    map.loading = false;
                                    map.load_error = Some(e.clone());
                                }
                                st.toast = Some(Toast::error(e));
                            }
                        }
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    pub fn update_map(
        &mut self,
        id: &str,
        f: impl FnOnce(&mut MapDocumentState) -> Result<(), String>,
    ) {
        if let Some(tab) = self.open_tabs.iter_mut().find(|t| t.id == id) {
            if let Some(map) = tab.map_state.as_mut().filter(|m| !m.loading) {
                if let Err(error) = f(map) {
                    self.toast = Some(Toast::error(error));
                }
                tab.recompute_modified();
            }
        }
    }
}

pub fn handle_map_key(
    event: &KeyDownEvent,
    state: &mut AppState,
    clipboard: Option<&str>,
) -> Option<bool> {
    state.active_map()?;
    if state.has_active_text_input() && state.active_map()?.input.is_none() {
        return None;
    }
    let key = event.keystroke.key.as_str();
    if state.active_map()?.input.is_some() {
        if key == "enter" || key == "escape" {
            let result = if key == "enter" {
                state.active_map_mut()?.apply_input()
            } else {
                state.active_map_mut()?.input = None;
                Ok(())
            };
            if let Err(e) = result {
                state.toast = Some(Toast::error(e));
            }
        } else {
            let mut cursor = state.text_input_cursor;
            let mut anchor = state.text_input_select_anchor;
            let map = state.active_map_mut()?;
            let target = if map.input == Some(crate::model::map_editor::MapInput::Search) {
                map.page = 0;
                &mut map.filter
            } else {
                &mut map.input_text
            };
            crate::ui::components::handle_text_key_event(
                target,
                &mut cursor,
                &mut anchor,
                event,
                clipboard,
            );
            state.text_input_cursor = cursor;
            state.text_input_select_anchor = anchor;
        }
    } else {
        let map = state.active_map_mut()?;
        if map.loading {
            return Some(true);
        }
        match (
            key,
            event.keystroke.modifiers.control,
            event.keystroke.modifiers.shift,
        ) {
            ("z", true, false) => map.undo(),
            ("y", true, _) | ("z", true, true) => map.redo(),
            ("d", true, _) => {
                let _ = map.duplicate();
            }
            ("delete", false, _) => {
                let _ = map.delete();
            }
            ("f", false, _) => map.focus_selected(),
            ("escape", false, _) => map.select(None),
            _ => return None,
        }
    }
    state.sync_active_tab_modified();
    Some(true)
}
