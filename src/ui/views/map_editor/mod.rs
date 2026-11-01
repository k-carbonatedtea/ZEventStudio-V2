mod async_render;
mod viewport;
use crate::{
    model::map_editor::{MapDocumentState, MapInput},
    ui::{state::AppState, theme::ThemePalette},
};
use gpui::*;
use roead::byml::Byml;
use std::{cell::RefCell, sync::Arc};
use uking_library::map::{actor_name, hash_id, Transform};

struct OutlinerMatches {
    root: Arc<Byml>,
    query: String,
    indices: Arc<Vec<usize>>,
}

thread_local! {
    static OUTLINER_MATCHES: RefCell<Option<OutlinerMatches>> = const { RefCell::new(None) };
}

fn actor_matches(actor: &Byml, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let name = actor_name(actor);
    let name_matches = if name.is_ascii() && query.is_ascii() {
        name.as_bytes()
            .windows(query.len())
            .any(|part| part.eq_ignore_ascii_case(query.as_bytes()))
    } else {
        name.to_lowercase().contains(query)
    };
    if name_matches {
        return true;
    }
    if query.len() > 8 || !query.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return false;
    }
    hash_id(actor).is_some_and(|hash| {
        let mut digits = [b'0'; 8];
        for (index, digit) in digits.iter_mut().enumerate() {
            *digit = b"0123456789abcdef"[((hash >> ((7 - index) * 4)) & 15) as usize];
        }
        digits
            .windows(query.len())
            .any(|part| part.eq_ignore_ascii_case(query.as_bytes()))
    })
}

fn filtered_actor_indices(map: &MapDocumentState, query: &str) -> Option<Arc<Vec<usize>>> {
    if query.is_empty() {
        return None;
    }
    OUTLINER_MATCHES.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(matches) = cache.as_ref() {
            if Arc::ptr_eq(&matches.root, &map.file.root) && matches.query == query {
                return Some(matches.indices.clone());
            }
        }
        let indices: Arc<Vec<usize>> = Arc::new(
            map.file
                .actors()
                .iter()
                .enumerate()
                .filter_map(|(index, actor)| actor_matches(actor, query).then_some(index))
                .collect(),
        );
        // Retaining the root prevents an allocation reused after edits or tab
        // switches from being mistaken for the cached document.
        *cache = Some(OutlinerMatches {
            root: map.file.root.clone(),
            query: query.into(),
            indices: indices.clone(),
        });
        Some(indices)
    })
}

#[cfg(test)]
mod outliner_tests {
    use super::{actor_matches, filtered_actor_indices, Arc, Byml, MapDocumentState};

    fn document() -> MapDocumentState {
        let root = Byml::from_text(
            "Objs:\n- {UnitConfigName: Obj_Tree, HashId: !u 43981}\n- {UnitConfigName: Rock, HashId: !u 1}",
        )
        .unwrap();
        let file = uking_library::map::MapFile::from_bytes(&root.to_binary(roead::Endian::Little))
            .unwrap();
        MapDocumentState::empty(file, "outliner-test", "outliner-test")
    }

    #[test]
    fn matches_actor_names_and_zero_padded_hash_substrings() {
        let map = document();
        let tree = &map.file.actors()[0];
        assert!(actor_matches(tree, "obj_tree"));
        assert!(actor_matches(tree, "TrEe"));
        assert!(actor_matches(tree, "0000ab"));
        assert!(actor_matches(tree, "ABCD"));
        assert!(!actor_matches(tree, "00000abcd"));
        assert!(!actor_matches(tree, "xyz"));
        assert!(!actor_matches(tree, "树"));
    }

    #[test]
    fn reuses_filtered_indices_until_query_or_document_changes() {
        let mut map = document();
        assert!(filtered_actor_indices(&map, "").is_none());
        let first = filtered_actor_indices(&map, "rock").unwrap();
        assert_eq!(first.as_slice(), &[1]);
        let cached = filtered_actor_indices(&map, "rock").unwrap();
        assert!(Arc::ptr_eq(&first, &cached));
        let trees = filtered_actor_indices(&map, "tree").unwrap();
        assert_eq!(trees.as_slice(), &[0]);
        map.file.delete(0).unwrap();
        let edited = filtered_actor_indices(&map, "rock").unwrap();
        assert_eq!(edited.as_slice(), &[0]);
        assert!(!Arc::ptr_eq(&first, &edited));
    }
}

pub fn button(
    label: impl Into<SharedString>,
    active: bool,
    pal: &ThemePalette,
    model: &Entity<AppState>,
    id: &str,
    action: impl Fn(&mut MapDocumentState) -> Result<(), String> + 'static,
) -> AnyElement {
    let model = model.clone();
    let id = id.to_string();
    div()
        .px_2()
        .py_1()
        .rounded_md()
        .text_xs()
        .cursor_pointer()
        .bg(if active {
            pal.primary_accent
        } else {
            pal.card_surface
        })
        .text_color(pal.text_primary)
        .hover(|s| s.opacity(0.8))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model.update(cx, |st, cx| {
                st.clear_active_text_inputs();
                st.update_map(&id, |m| action(m));
                cx.notify();
            });
        })
        .child(label.into())
        .into_any_element()
}

pub fn render_map_editor(state: &AppState, model: &Entity<AppState>) -> AnyElement {
    let pal = state.theme();
    let Some(map) = state.active_map() else {
        return div().into_any_element();
    };
    let id = state.active_tab_id.as_deref().unwrap_or("");
    let m = model.clone();
    let save_id = id.to_string();
    let missing = map
        .assets
        .actors
        .values()
        .filter(|v| v.diagnostic.is_some())
        .count();
    let toolbar = div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_2()
        .border_b_1()
        .border_color(pal.card_border)
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_sm()
                .child("地图编辑器"),
        )
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(pal.text_secondary)
                .child(format!(
                    "{} 对象 · {} 地形块 · {} 三角形",
                    map.file.actors().len(),
                    map.assets.heights.len(),
                    map.scene.total_triangles
                )),
        )
        .child(button("撤销", false, &pal, model, id, |m| {
            m.undo();
            Ok(())
        }))
        .child(button("重做", false, &pal, model, id, |m| {
            m.redo();
            Ok(())
        }))
        .child(button("全景", false, &pal, model, id, |m| {
            m.scene.fit_camera();
            Ok(())
        }))
        .child(button("俯视", false, &pal, model, id, |m| {
            m.scene
                .camera
                .set_preset(crate::model::model_preview::PreviewPresetView::Top);
            Ok(())
        }))
        .child(
            div()
                .px_3()
                .py_1()
                .rounded_md()
                .bg(pal.primary_accent)
                .text_xs()
                .cursor_pointer()
                .child("保存 Ctrl+S")
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    m.update(cx, |st, cx| {
                        st.save_document_tab(&save_id);
                        cx.notify();
                    });
                }),
        );
    let mut layers = div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_1()
        .bg(pal.panel_surface)
        .child(button("模型", map.show_models, &pal, model, id, |m| {
            m.show_models = !m.show_models;
            m.rebuild_instances();
            Ok(())
        }))
        .child(button("地形", map.show_terrain, &pal, model, id, |m| {
            m.show_terrain = !m.show_terrain;
            m.rebuild_instances();
            Ok(())
        }))
        .child(button("水体", map.show_water, &pal, model, id, |m| {
            m.show_water = !m.show_water;
            m.rebuild_instances();
            Ok(())
        }))
        .child(button(
            "区域 / 标记",
            map.show_tags,
            &pal,
            model,
            id,
            |m| {
                m.show_tags = !m.show_tags;
                m.rebuild_instances();
                Ok(())
            },
        ))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(if missing > 0 {
                    rgb(0xE9AA61).into()
                } else {
                    pal.text_secondary
                })
                .child(format!("{missing} 类资源待检查")),
        );
    if let Some(path) = &map.assets.context_path {
        let path = path.clone();
        let model = model.clone();
        layers = layers.child(
            div()
                .px_2()
                .py_1()
                .text_xs()
                .cursor_pointer()
                .rounded_md()
                .bg(pal.card_surface)
                .child(format!(
                    "配套地图：{} 对象 · 点击编辑",
                    map.assets.context_actors.len()
                ))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model.update(cx, |st, cx| {
                        st.clear_active_text_inputs();
                        st.load_file_by_path(&path);
                        cx.notify();
                    });
                }),
        );
    }
    div().size_full().flex().flex_col().overflow_hidden().bg(pal.app_bg).text_color(pal.text_primary)
        .child(toolbar).child(layers)
        .child(div().flex_1().min_h_0().flex().overflow_hidden()
            .child(outliner(map,state,model,id))
            .child(viewport::render_viewport(map,model,id))
            .child(inspector(map,state,model,id)))
        .child(div().px_3().py_1().bg(pal.panel_surface).text_xs().text_color(pal.text_secondary)
            .child("左键选择 · 右键旋转 · 中键平移 · 滚轮缩放 · Shift+左拖移动 · F 聚焦 · Ctrl+D 复制 · Delete 删除"))
        .into_any_element()
}

fn outliner(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
) -> AnyElement {
    let pal = state.theme();
    let model_search = model.clone();
    let tid = id.to_string();
    let query = map.filter.to_lowercase();
    let actors = map.file.actors();
    let filtered = filtered_actor_indices(map, &query);
    let count = filtered
        .as_ref()
        .map_or(actors.len(), |indices| indices.len());
    let pages = count.div_ceil(100).max(1);
    let page = map.page.min(pages - 1);
    div()
        .w(px(200.))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .border_r_1()
        .border_color(pal.card_border)
        .bg(pal.panel_surface)
        .child(
            div()
                .px_3()
                .py_2()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .child(format!("场景对象 ({count})")),
        )
        .child(
            div()
                .mx_2()
                .mb_2()
                .p_2()
                .rounded_md()
                .border_1()
                .border_color(if map.input == Some(MapInput::Search) {
                    pal.primary_accent
                } else {
                    pal.card_border
                })
                .text_xs()
                .cursor_text()
                .child(if map.filter.is_empty() {
                    "搜索 Actor / HashId".into()
                } else {
                    map.filter.clone()
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_search.update(cx, |st, cx| {
                        st.clear_active_text_inputs();
                        if let Some(m) = st.active_map_mut() {
                            m.input = Some(MapInput::Search);
                            st.text_input_cursor = m.filter.chars().count();
                        }
                        cx.notify();
                    });
                    let _ = &tid;
                }),
        )
        .child(
            div()
                .id("map-outliner")
                .flex_1()
                .overflow_y_scroll()
                .children((page * 100..(page * 100 + 100).min(count)).map(|position| {
                    let i = filtered
                        .as_ref()
                        .map_or(position, |indices| indices[position]);
                    let actor = &actors[i];
                    let selected = map.selected == Some(i);
                    let name = actor_name(actor).to_string();
                    let diagnostic = map
                        .assets
                        .actors
                        .get(&name)
                        .is_some_and(|v| v.diagnostic.is_some());
                    let model = model.clone();
                    let id = id.to_string();
                    div()
                        .px_2()
                        .py_1()
                        .cursor_pointer()
                        .bg(if selected {
                            pal.card_hover
                        } else {
                            pal.panel_surface
                        })
                        .hover(|s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                            model.update(cx, |st, cx| {
                                st.clear_active_text_inputs();
                                st.update_map(&id, |m| {
                                    m.select(Some(i));
                                    if e.click_count > 1 {
                                        m.focus_selected();
                                    }
                                    Ok(())
                                });
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_xs()
                                .text_color(if diagnostic {
                                    rgb(0xE9AA61).into()
                                } else {
                                    pal.text_primary
                                })
                                .overflow_hidden()
                                .child(format!("{} {name}", if diagnostic { "!" } else { "·" })),
                        )
                        .child(
                            div()
                                .text_size(px(10.))
                                .text_color(pal.text_secondary)
                                .child(format!("{:08X}", hash_id(actor).unwrap_or(0))),
                        )
                })),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p_2()
                .child(button("‹", false, &pal, model, id, |m| {
                    m.page = m.page.saturating_sub(1);
                    Ok(())
                }))
                .child(div().text_xs().child(format!("{} / {pages}", page + 1)))
                .child(button("›", false, &pal, model, id, move |m| {
                    m.page = (m.page + 1).min(pages - 1);
                    Ok(())
                })),
        )
        .into_any_element()
}

fn inspector(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
) -> AnyElement {
    let pal = state.theme();
    let mut panel = div()
        .id("map-inspector")
        .w(px(252.))
        .flex_shrink_0()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .overflow_y_scroll()
        .border_l_1()
        .border_color(pal.card_border)
        .bg(pal.panel_surface);
    if let Some(error) = &map.load_error {
        panel = panel.child(
            div()
                .text_xs()
                .text_color(rgb(0xED8B71))
                .child(error.clone()),
        );
    }
    if !map.assets.background_meshes.is_empty() {
        panel = panel.child(div().text_xs().text_color(pal.text_secondary).child(
            "神庙主体为合并背景。缺少独立模型的建筑仅编辑放置数据，合并模型与碰撞不会重建。",
        ));
    }
    for warning in &map.assets.warnings {
        panel = panel.child(
            div()
                .text_xs()
                .text_color(rgb(0xE9AA61))
                .child(warning.clone()),
        );
    }
    let Some((index, actor)) = map
        .selected
        .and_then(|i| map.file.actors().get(i).map(|a| (i, a)))
    else {
        return panel
            .child(
                div()
                    .text_sm()
                    .text_color(pal.text_secondary)
                    .child("选择对象以编辑变换与查看模型来源"),
            )
            .into_any_element();
    };
    panel = panel
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .child(actor_name(actor).to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(pal.text_secondary)
                .child(format!(
                    "HashId {:08X} · #{}",
                    hash_id(actor).unwrap_or(0),
                    index
                )),
        );
    if let Some(visual) = map.assets.actors.get(actor_name(actor)) {
        panel = panel.child(
            div()
                .text_xs()
                .text_color(pal.text_secondary)
                .child(visual.resource.clone()),
        );
        if let Some(error) = &visual.diagnostic {
            panel = panel.child(
                div()
                    .text_xs()
                    .text_color(rgb(0xE9AA61))
                    .child(error.clone()),
            );
        }
    }
    let t = Transform::from_actor(actor);
    for (kind, title, values) in [
        (0, "位置", t.translation),
        (1, "旋转 (°)", t.rotation.map(f32::to_degrees)),
        (2, "缩放", t.scale),
    ] {
        panel = panel.child(
            div()
                .mt_2()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .child(title),
        );
        for axis in 0..3 {
            let value = values[axis];
            let m = model.clone();
            let id = id.to_string();
            let field = MapInput::Transform(kind, axis);
            let editing = map.input == Some(field);
            panel = panel.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(14.))
                            .text_xs()
                            .text_color([rgb(0xED7C73), rgb(0x8DC87A), rgb(0x76A9EA)][axis])
                            .child(["X", "Y", "Z"][axis]),
                    )
                    .child(
                        div()
                            .flex_1()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .border_1()
                            .border_color(if editing {
                                pal.primary_accent
                            } else {
                                pal.card_border
                            })
                            .bg(pal.card_surface)
                            .cursor_text()
                            .text_xs()
                            .child(if editing {
                                format!("{} ▏", map.input_text)
                            } else {
                                format!("{value:.4}")
                            })
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                m.update(cx, |st, cx| {
                                    st.clear_active_text_inputs();
                                    st.update_map(&id, |map| {
                                        map.input = Some(field);
                                        map.input_text = format!("{value:.4}");
                                        Ok(())
                                    });
                                    st.text_input_cursor =
                                        st.active_map().map_or(0, |m| m.input_text.chars().count());
                                    st.text_input_select_anchor = Some(0);
                                    cx.notify();
                                });
                            }),
                    ),
            );
        }
    }
    panel = panel
        .child(
            div()
                .text_size(px(10.))
                .text_color(pal.text_secondary)
                .child("输入数值后按 Enter 应用"),
        )
        .child(button("聚焦对象 (F)", false, &pal, model, id, |m| {
            m.focus_selected();
            Ok(())
        }))
        .child(button("吸附到地形", false, &pal, model, id, |m| {
            m.snap_to_terrain()
        }))
        .child(
            div()
                .flex()
                .gap_2()
                .child(button("复制", false, &pal, model, id, |m| m.duplicate()))
                .child(button("删除", false, &pal, model, id, |m| m.delete())),
        );
    if let Some(links) = actor
        .as_map()
        .ok()
        .and_then(|m| m.get("LinksToObj"))
        .and_then(|v| v.as_array().ok())
    {
        panel = panel.child(
            div()
                .mt_2()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .child(format!("对象连线 ({})", links.len())),
        );
        for link in links {
            if let Ok(m) = link.as_map() {
                let name = m
                    .get("DefinitionName")
                    .and_then(|v| v.as_string().ok())
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let dest = m.get("DestUnitHashId").and_then(|v| match v {
                    roead::byml::Byml::U32(v) => Some(*v),
                    roead::byml::Byml::I32(v) => Some(*v as u32),
                    _ => None,
                });
                if let Some(dest) = dest {
                    let target = map
                        .file
                        .actors()
                        .iter()
                        .position(|a| hash_id(a) == Some(dest));
                    panel = panel.child(button(
                        format!("{name} → {dest:08X}"),
                        false,
                        &pal,
                        model,
                        id,
                        move |m| {
                            m.select(target);
                            m.focus_selected();
                            Ok(())
                        },
                    ));
                }
            }
        }
    }
    panel.into_any_element()
}
