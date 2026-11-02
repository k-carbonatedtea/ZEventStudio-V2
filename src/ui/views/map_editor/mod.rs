mod async_render;
mod inspector;
mod outliner;
mod viewport;
use crate::{
    model::map_editor::{MapDocumentState, MapViewMode},
    ui::{state::AppState, theme::ThemePalette},
};
use gpui::*;
use roead::byml::Byml;
use uking_library::map::{actor_name, hash_id};

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

#[cfg(test)]
mod outliner_tests {
    use super::{actor_matches, outliner::rows, Byml, MapDocumentState};
    use crate::model::map_editor::ActorRef;
    use std::sync::Arc;

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
        map.filter = "rock".into();
        let first = rows(&map);
        assert_eq!(first.as_slice(), &[ActorRef::Current(1)]);
        assert!(Arc::ptr_eq(&first, &rows(&map)));
        map.filter = "tree".into();
        assert_eq!(rows(&map).as_slice(), &[ActorRef::Current(0)]);
        map.file.delete(0).unwrap();
        map.filter = "rock".into();
        let edited = rows(&map);
        assert_eq!(edited.as_slice(), &[ActorRef::Current(0)]);
        assert!(!Arc::ptr_eq(&first, &edited));
        Arc::make_mut(&mut map.assets)
            .context_actors
            .push(Byml::from_text("{UnitConfigName: Rock, HashId: !u 2}").unwrap());
        map.outliner_scope = 1;
        assert_eq!(rows(&map).as_slice(), &[ActorRef::Context(0)]);
        map.outliner_scope = 2;
        assert_eq!(
            rows(&map).as_slice(),
            &[ActorRef::Current(0), ActorRef::Context(0)]
        );
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
                if st
                    .active_map()
                    .is_some_and(|m| m.input.is_some() && m.input_error.is_some())
                {
                    return;
                }
                st.update_map(&id, |m| {
                    let result = action(m);
                    m.input_error = result.as_ref().err().cloned();
                    result
                });
                if let Some(map) = st
                    .active_map()
                    .filter(|m| m.input == Some(crate::model::map_editor::MapInput::PropertyName))
                {
                    st.text_input_cursor = map.input_text.chars().count();
                    st.text_input_select_anchor = Some(0);
                }
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
    let resize_model = model.clone();
    let resize_id = id.to_string();
    let up_model = model.clone();
    let up_id = id.to_string();
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
        .flex_wrap()
        .items_center()
        .gap_2()
        .px_3()
        .py_1()
        .bg(pal.panel_surface)
        .children(MapViewMode::ALL.into_iter().map(|mode| {
            button(
                mode.label(),
                map.view_mode == mode,
                &pal,
                model,
                id,
                move |m| {
                    m.set_view_mode(mode);
                    Ok(())
                },
            )
        }))
        .child(button(
            "可见对象",
            map.show_models,
            &pal,
            model,
            id,
            |m| {
                m.show_models = !m.show_models;
                m.rebuild_instances();
                Ok(())
            },
        ))
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
            "全部辅助对象",
            map.show_tags,
            &pal,
            model,
            id,
            |m| {
                m.toggle_tags();
                Ok(())
            },
        ))
        .child(button(
            "隐藏对象透视",
            map.show_hidden_perspective,
            &pal,
            model,
            id,
            |m| {
                m.show_hidden_perspective = !m.show_hidden_perspective;
                Ok(())
            },
        ))
        .child(button(
            "对象链接",
            map.show_links,
            &pal,
            model,
            id,
            |m| {
                m.toggle_links();
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
        .on_mouse_move(move |e,_,cx| {resize_model.update(cx,|st,cx| {
            if st.active_map().is_some_and(|m|m.panel_resize.is_some()) {
                st.update_map(&resize_id,|m| {
                    if !e.dragging() {m.panel_resize=None;return Ok(());}
                    if let Some((right,start,width))=m.panel_resize {
                    let delta=f32::from(e.position.x)-start;
                    if right {m.inspector_width=(width-delta).clamp(300.,700.);}else{m.outliner_width=(width+delta).clamp(180.,420.);}
                } Ok(())});cx.notify();
            }
        });})
        .on_mouse_up(MouseButton::Left,move |_,_,cx| {up_model.update(cx,|st,cx| {st.update_map(&up_id,|m|{m.panel_resize=None;Ok(())});cx.notify();});})
        .child(toolbar).child(layers)
        .child(div().flex_1().min_h_0().flex().overflow_hidden()
            .child(outliner::render(map,state,model,id))
            .child(resize_handle(false,map,state,model,id))
            .child(viewport::render_viewport(map,model,id))
            .child(resize_handle(true,map,state,model,id))
            .child(inspector::render(map,state,model,id)))
        .child(div().px_3().py_1().bg(pal.panel_surface).text_xs().text_color(pal.text_secondary)
            .child("左键选择 · 右键旋转 · 中键平移 · 滚轮缩放 · Shift+左拖移动 · F 聚焦 · Ctrl+D 复制 · Delete 删除"))
        .into_any_element()
}

fn resize_handle(
    right: bool,
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
) -> AnyElement {
    let pal = state.theme();
    let model = model.clone();
    let id = id.to_string();
    let width = if right {
        map.inspector_width
    } else {
        map.outliner_width
    };
    div()
        .w(px(4.))
        .h_full()
        .flex_shrink_0()
        .cursor_col_resize()
        .bg(pal.card_border)
        .hover(|s| s.bg(pal.primary_accent))
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            cx.stop_propagation();
            model.update(cx, |st, cx| {
                st.update_map(&id, |m| {
                    m.panel_resize = Some((right, e.position.x.into(), width));
                    Ok(())
                });
                cx.notify();
            });
        })
        .into_any_element()
}
