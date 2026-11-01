use super::{actor_matches, button, inspector::section};
use crate::{
    model::map_editor::{ActorRef, MapDocumentState, MapInput},
    ui::state::AppState,
};
use gpui::*;
use roead::byml::Byml;
use std::{cell::RefCell, sync::Arc};
use uking_library::map::{actor_name, hash_id};

struct Matches {
    root: Arc<Byml>,
    assets: Arc<crate::model::map_editor::MapAssets>,
    query: String,
    scope: usize,
    rows: Arc<Vec<ActorRef>>,
}
thread_local! {static MATCHES:RefCell<Option<Matches>> = const {RefCell::new(None)};}
pub(super) fn rows(map: &MapDocumentState) -> Arc<Vec<ActorRef>> {
    MATCHES.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(c) = cache.as_ref() {
            if Arc::ptr_eq(&c.root, &map.file.root)
                && Arc::ptr_eq(&c.assets, &map.assets)
                && c.query == map.filter
                && c.scope == map.outliner_scope
            {
                return c.rows.clone();
            }
        }
        let query = map.filter.to_lowercase();
        let rows: Arc<Vec<ActorRef>> = Arc::new(
            map.file
                .actors()
                .iter()
                .enumerate()
                .filter(|(_, a)| map.outliner_scope != 1 && actor_matches(a, &query))
                .map(|(i, _)| ActorRef::Current(i))
                .chain(
                    map.assets
                        .context_actors
                        .iter()
                        .enumerate()
                        .filter(|(_, a)| map.outliner_scope != 0 && actor_matches(a, &query))
                        .map(|(i, _)| ActorRef::Context(i)),
                )
                .collect(),
        );
        *cache = Some(Matches {
            root: map.file.root.clone(),
            assets: map.assets.clone(),
            query: map.filter.clone(),
            scope: map.outliner_scope,
            rows: rows.clone(),
        });
        rows
    })
}
pub fn render(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
) -> AnyElement {
    let pal = state.theme();
    let entries = rows(map);
    let count = entries.len();
    let snapshot = map.clone();
    let list_model = model.clone();
    let tid = id.to_string();
    let search_model = model.clone();
    let search_id = id.to_string();
    let mut controls = div().flex().flex_col().gap_1();
    for (label, active, kind) in [
        ("显示可见对象", map.show_models, 0),
        ("显示隐藏对象", map.show_tags, 1),
        ("显示地图模型", map.show_map_models, 2),
        ("显示配套地图", map.show_context, 3),
        ("显示对象链接", map.show_links, 4),
        ("仅选中对象的链接", map.selected_links_only, 5),
        ("显示地形", map.show_terrain, 6),
        ("显示水体", map.show_water, 7),
    ] {
        controls = controls.child(button(
            format!("{}  {label}", if active { "☑" } else { "☐" }),
            false,
            &pal,
            model,
            id,
            move |m| {
                match kind {
                    0 => m.show_models = !m.show_models,
                    1 => m.show_tags = !m.show_tags,
                    2 => m.show_map_models = !m.show_map_models,
                    3 => m.show_context = !m.show_context,
                    4 => m.show_links = !m.show_links,
                    5 => m.selected_links_only = !m.selected_links_only,
                    6 => m.show_terrain = !m.show_terrain,
                    _ => m.show_water = !m.show_water,
                }
                if !matches!(kind, 4 | 5) {
                    m.rebuild_instances();
                }
                Ok(())
            },
        ));
    }
    div()
        .w(px(map.outliner_width))
        .flex_shrink_0()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(pal.panel_surface)
        .child(section(
            map,
            state,
            model,
            id,
            "project-resources",
            "项目资源",
            div()
                .h(px(180.))
                .child(crate::ui::components::render_file_tree(
                    &state.file_tree,
                    state,
                    model,
                    0,
                ))
                .into_any_element(),
        ))
        .child(
            div()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(pal.card_border)
                .font_weight(FontWeight::BOLD)
                .text_xs()
                .child(format!("大纲 · {count} 对象")),
        )
        .child(
            div().p_2().flex().gap_1().children(
                [("当前", 0), ("配套", 1), ("全部", 2)]
                    .into_iter()
                    .map(|(label, scope)| {
                        button(
                            label,
                            map.outliner_scope == scope,
                            &pal,
                            model,
                            id,
                            move |m| {
                                m.outliner_scope = scope;
                                Ok(())
                            },
                        )
                    }),
            ),
        )
        .child(
            div()
                .mx_2()
                .mb_2()
                .px_2()
                .py_1()
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
                    div()
                        .text_color(pal.text_secondary)
                        .child("搜索 Actor / HashId")
                        .into_any_element()
                } else {
                    crate::ui::components::render_input_text(
                        &map.filter,
                        map.input == Some(MapInput::Search),
                        state,
                        model,
                    )
                    .into_any_element()
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    search_model.update(cx, |st, cx| {
                        if st
                            .active_map()
                            .is_some_and(|m| m.input == Some(MapInput::Search))
                        {
                            return;
                        }
                        st.clear_active_text_inputs();
                        if st
                            .active_map()
                            .is_some_and(|m| m.input.is_some() && m.input_error.is_some())
                        {
                            return;
                        }
                        st.update_map(&search_id, |m| {
                            m.input = Some(MapInput::Search);
                            Ok(())
                        });
                        st.text_input_cursor =
                            st.active_map().map_or(0, |m| m.filter.chars().count());
                        cx.notify();
                    });
                }),
        )
        .child(
            uniform_list("map-object-list", count, move |range, _, _| {
                range
                    .map(|i| {
                        let reference = entries[i];
                        let actor = snapshot.actor(reference).unwrap();
                        let name = actor_name(actor);
                        let selected = snapshot.selected_ref() == Some(reference);
                        let visible = snapshot.object_visible(reference);
                        let hidden = snapshot.hidden_objects.contains(&reference);
                        let logical = snapshot
                            .assets
                            .actors
                            .get(name)
                            .is_some_and(|v| !v.has_model);
                        let missing = snapshot
                            .assets
                            .actors
                            .get(name)
                            .is_some_and(|v| v.diagnostic.is_some());
                        let m = list_model.clone();
                        let tab = tid.clone();
                        let toggle_model = list_model.clone();
                        let toggle_id = tid.clone();
                        div()
                            .h(px(44.))
                            .px_2()
                            .flex()
                            .items_center()
                            .gap_2()
                            .cursor_pointer()
                            .bg(if selected {
                                pal.card_hover
                            } else {
                                pal.panel_surface
                            })
                            .hover(|s| s.bg(pal.card_hover))
                            .child(
                                div()
                                    .w(px(22.))
                                    .text_color(if hidden {
                                        pal.text_secondary
                                    } else {
                                        pal.primary_accent
                                    })
                                    .child(if hidden { "☐" } else { "☑" })
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        cx.stop_propagation();
                                        toggle_model.update(cx, |st, cx| {
                                            st.update_map(&toggle_id, |m| {
                                                m.toggle_object_visibility(reference);
                                                Ok(())
                                            });
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .opacity(if visible { 1. } else { 0.5 })
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if missing {
                                                rgb(0xE9AA61).into()
                                            } else {
                                                pal.text_primary
                                            })
                                            .child(format!(
                                                "{}{}",
                                                if missing {
                                                    "! "
                                                } else if logical {
                                                    "◇ "
                                                } else {
                                                    ""
                                                },
                                                name
                                            )),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(pal.text_secondary)
                                            .child(format!(
                                                "{:08X} · {}",
                                                hash_id(actor).unwrap_or(0),
                                                if matches!(reference, ActorRef::Context(_)) {
                                                    "配套 · 只读"
                                                } else {
                                                    "当前"
                                                }
                                            )),
                                    ),
                            )
                            .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                                m.update(cx, |st, cx| {
                                    st.clear_active_text_inputs();
                                    if st.active_map().is_some_and(|m| {
                                        m.input.is_some() && m.input_error.is_some()
                                    }) {
                                        return;
                                    }
                                    st.update_map(&tab, |m| {
                                        m.select_ref(Some(reference));
                                        if e.click_count > 1 {
                                            m.focus_selected();
                                        }
                                        Ok(())
                                    });
                                    cx.notify();
                                });
                            })
                            .into_any_element()
                    })
                    .collect()
            })
            .flex_1()
            .min_h_0(),
        )
        .children((count == 0).then(|| {
            div()
                .p_3()
                .text_xs()
                .text_color(pal.text_secondary)
                .child("没有匹配的对象")
        }))
        .child(section(
            map,
            state,
            model,
            id,
            "display",
            "工具 / 对象显示",
            controls.into_any_element(),
        ))
        .into_any_element()
}
