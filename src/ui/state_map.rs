use crate::{
    model::map_editor::MapDocumentState,
    ui::{components::Toast, state::AppState},
};
use gpui::*;

impl AppState {
    /// New placements load only their missing model in the background. Merge
    /// resource additions without replacing newer edits, selection or camera.
    pub fn start_pending_map_assets(&mut self, cx: &mut Context<Self>) {
        use std::sync::Arc;
        let Some(id) = self.active_tab_id.clone() else {
            return;
        };
        let settings = self.game_paths.clone();
        let project = self.project_path.clone();
        if let Some(map) = self
            .active_map_mut()
            .filter(|map| !map.loading && map.asset_loading.is_none())
        {
            let name = loop {
                let Some(name) = map.asset_queue.pop_front() else {
                    break None;
                };
                let still_used = map
                    .file
                    .actors()
                    .iter()
                    .any(|actor| uking_library::map::actor_name(actor) == name);
                if still_used
                    && !map.assets.actors.contains_key(&name)
                    && !map.asset_failed.contains(&name)
                {
                    break Some(name);
                }
            };
            if let Some(name) = name {
                map.asset_loading = Some(name.clone());
                let snapshot = map.clone();
                let meshes = snapshot.scene.meshes.clone();
                let assets = snapshot.assets.clone();
                let id = id.clone();
                let settings = settings.clone();
                let project = project.clone();
                cx.spawn(async move |this, cx| {
                    let requested_name = name.clone();
                    let result = cx
                        .background_spawn(async move {
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                crate::model::map_editor::loader::load_actor_asset(
                                    snapshot,
                                    settings,
                                    project,
                                    &requested_name,
                                )
                            }))
                            .unwrap_or_else(|_| Err("资产模型解析异常".into()))
                        })
                        .await;
                    let _ = this.update(cx, |st, cx| {
                        if let Some(map) = st
                            .open_tabs
                            .iter_mut()
                            .find(|tab| tab.id == id)
                            .and_then(|tab| tab.map_state.as_mut())
                        {
                            if map.asset_loading.as_deref() == Some(&name) {
                                map.asset_loading = None;
                                if !map.loading
                                    && Arc::ptr_eq(&meshes, &map.scene.meshes)
                                    && Arc::ptr_eq(&assets, &map.assets)
                                {
                                    match result {
                                        Ok(loaded) => {
                                            map.scene.meshes = loaded.scene.meshes;
                                            map.scene.mesh_items = loaded.scene.mesh_items;
                                            map.scene.textures = loaded.scene.textures;
                                            map.assets = loaded.assets;
                                            map.rebuild_instances();
                                        }
                                        Err(error) => {
                                            Arc::make_mut(&mut map.asset_failed).insert(name);
                                            map.input_error = Some(error);
                                        }
                                    }
                                } else if map
                                    .file
                                    .actors()
                                    .iter()
                                    .any(|actor| uking_library::map::actor_name(actor) == name)
                                    && !map.assets.actors.contains_key(&name)
                                    && !map.asset_failed.contains(&name)
                                {
                                    map.asset_queue.push_front(name.clone());
                                }
                                cx.notify();
                            }
                        }
                    });
                })
                .detach();
            }
        }
        let Some(map) = self
            .active_map_mut()
            .filter(|map| !map.loading && map.asset_browser.open)
        else {
            return;
        };
        let Some(name) = map.asset_browser.selected.clone() else {
            return;
        };
        if map
            .asset_preview
            .as_ref()
            .is_some_and(|preview| preview.name == name)
        {
            return;
        }
        let source = map.scene.file_path.clone();
        let big_endian = map.file.big_endian;
        let catalog = map.assets.catalog.clone();
        map.asset_preview = Some(crate::model::map_editor::AssetPreviewState {
            name: name.clone(),
            loading: true,
            image: None,
            error: None,
        });
        cx.spawn(async move |this, cx| {
            let requested_name = name.clone();
            let result = cx
                .background_spawn(async move {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let scene = crate::model::map_editor::loader::load_asset_preview(
                            &source,
                            big_endian,
                            settings,
                            project,
                            &requested_name,
                        )?;
                        crate::ui::views::model_preview::rasterizer::rasterize_scene(
                            &scene, 256., 144.,
                        )
                        .ok_or_else(|| "模型预览没有可绘制内容".to_string())
                    }))
                    .unwrap_or_else(|_| Err("资产预览解析异常".into()))
                })
                .await;
            let _ = this.update(cx, |st, cx| {
                if let Some(map) = st
                    .open_tabs
                    .iter_mut()
                    .find(|tab| tab.id == id)
                    .and_then(|tab| tab.map_state.as_mut())
                {
                    if map
                        .asset_preview
                        .as_ref()
                        .is_some_and(|preview| preview.loading && preview.name == name)
                    {
                        map.asset_preview = if map.asset_browser.selected.as_deref() == Some(&name)
                            && Arc::ptr_eq(&catalog, &map.assets.catalog)
                        {
                            let (image, error) = match result {
                                Ok(image) => (Some(image), None),
                                Err(error) => (None, Some(error)),
                            };
                            Some(crate::model::map_editor::AssetPreviewState {
                                name,
                                loading: false,
                                image,
                                error,
                            })
                        } else {
                            None
                        };
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

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
                state.active_map_mut()?.input_error = None;
                Ok(())
            };
            if let Err(e) = result {
                state.toast = Some(Toast::error(e));
            }
        } else {
            let mut cursor = state.text_input_cursor;
            let mut anchor = state.text_input_select_anchor;
            let map = state.active_map_mut()?;
            let target = match map.input {
                Some(crate::model::map_editor::MapInput::Search) => {
                    map.page = 0;
                    &mut map.filter
                }
                Some(crate::model::map_editor::MapInput::AssetSearch) => {
                    &mut map.asset_browser.query
                }
                _ => &mut map.input_text,
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
                if let Err(error) = map.duplicate() {
                    map.input_error = Some(error.clone());
                    state.toast = Some(Toast::error(error));
                }
            }
            ("delete", false, _) => {
                if let Err(error) = map.delete() {
                    map.input_error = Some(error.clone());
                    state.toast = Some(Toast::error(error));
                }
            }
            ("f", false, _) => map.focus_selected(),
            ("escape", false, _) => {
                if map.asset_browser.pending_placement.take().is_none() {
                    map.select(None);
                }
            }
            _ => return None,
        }
    }
    state.sync_active_tab_modified();
    Some(true)
}
