use super::button;
use crate::{
    model::map_editor::{catalog::AssetCatalog, MapDocumentState, MapInput},
    ui::{state::AppState, theme::ThemePalette},
};
use gpui::*;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Clone)]
pub struct AssetDrag {
    pub name: String,
    pub tab_id: String,
}

impl Render for AssetDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(rgb(0x242128))
            .border_1()
            .border_color(rgb(0xF59E0B))
            .text_color(rgb(0xF5E9D7))
            .text_xs()
            .child(cube_icon(28.))
            .child(self.name.clone())
    }
}

struct FilterCache {
    catalog: Arc<AssetCatalog>,
    favorites: Arc<HashSet<String>>,
    query: String,
    category: Option<String>,
    favorites_only: bool,
    entries: Arc<Vec<usize>>,
}

thread_local! {
    static FILTER: RefCell<Option<FilterCache>> = const { RefCell::new(None) };
    static GRID_WIDTH: RefCell<HashMap<String, f32>> = RefCell::new(HashMap::new());
}

fn filtered_entries(map: &MapDocumentState) -> Arc<Vec<usize>> {
    FILTER.with(|cache| {
        let mut cache = cache.borrow_mut();
        let browser = &map.asset_browser;
        if let Some(cached) = cache.as_ref() {
            if Arc::ptr_eq(&cached.catalog, &map.assets.catalog)
                && Arc::ptr_eq(&cached.favorites, &browser.favorites)
                && cached.query == browser.query
                && cached.category == browser.category
                && cached.favorites_only == browser.favorites_only
            {
                return cached.entries.clone();
            }
        }
        let entries = Arc::new(map.assets.catalog.filtered_indices(
            &browser.query,
            browser.category.as_deref(),
            browser.favorites_only,
            &browser.favorites,
        ));
        *cache = Some(FilterCache {
            catalog: map.assets.catalog.clone(),
            favorites: browser.favorites.clone(),
            query: browser.query.clone(),
            category: browser.category.clone(),
            favorites_only: browser.favorites_only,
            entries: entries.clone(),
        });
        entries
    })
}

/// A small code-native asset cube, matching the browser's reusable icon system.
fn cube_icon(size: f32) -> AnyElement {
    canvas(
        |_, _, _| {},
        |bounds, _, window, _| {
            let x = f32::from(bounds.origin.x);
            let y = f32::from(bounds.origin.y);
            let w = f32::from(bounds.size.width);
            let h = f32::from(bounds.size.height);
            let p = [
                [0.5, 0.08],
                [0.91, 0.28],
                [0.91, 0.72],
                [0.5, 0.93],
                [0.09, 0.72],
                [0.09, 0.28],
                [0.5, 0.49],
            ];
            let mut path = PathBuilder::stroke(px(2.));
            for (a, b) in [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 4),
                (4, 5),
                (5, 0),
                (5, 6),
                (1, 6),
                (6, 3),
            ] {
                path.move_to(point(px(x + p[a][0] * w), px(y + p[a][1] * h)));
                path.line_to(point(px(x + p[b][0] * w), px(y + p[b][1] * h)));
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(0xF59E0B));
            }
        },
    )
    .size(px(size))
    .into_any_element()
}

fn field(label: &str, value: String, pal: &ThemePalette) -> AnyElement {
    div()
        .flex()
        .gap_2()
        .min_w_0()
        .child(
            div()
                .w(px(38.))
                .flex_shrink_0()
                .text_color(pal.text_secondary)
                .child(label.to_string()),
        )
        .child(div().flex_1().min_w_0().child(value))
        .into_any_element()
}

fn details(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
) -> AnyElement {
    let pal = state.theme();
    let mut detail = div()
        .id("asset-details")
        .w(px(245.))
        .flex_shrink_0()
        .border_l_1()
        .border_color(pal.card_border)
        .p_2()
        .text_xs()
        .flex()
        .flex_col()
        .gap_1()
        .overflow_y_scroll();
    let Some(asset) = map
        .asset_browser
        .selected
        .as_deref()
        .and_then(|name| map.assets.catalog.find(name))
    else {
        return detail
            .child(div().font_weight(FontWeight::BOLD).child("资产详情"))
            .child(
                div()
                    .mt_2()
                    .text_color(pal.text_secondary)
                    .child("选择资产查看模型与信息。"),
            )
            .child(
                div()
                    .text_color(pal.text_secondary)
                    .child("拖入场景或双击资产，再点击地图放置。"),
            )
            .into_any_element();
    };
    let preview = match map.asset_preview.as_ref().filter(|p| p.name == asset.name) {
        Some(preview) if preview.loading => div()
            .text_color(pal.text_secondary)
            .child("正在加载预览…")
            .into_any_element(),
        Some(preview) => {
            if let Some(image) = &preview.image {
                img(image.clone())
                    .size_full()
                    .object_fit(ObjectFit::Contain)
                    .into_any_element()
            } else {
                div()
                    .text_color(pal.text_secondary)
                    .child(
                        preview
                            .error
                            .clone()
                            .unwrap_or_else(|| "暂无模型预览".into()),
                    )
                    .into_any_element()
            }
        }
        None => div()
            .text_color(pal.text_secondary)
            .child("准备预览…")
            .into_any_element(),
    };
    detail = detail
        .child(
            div()
                .h(px(90.))
                .w_full()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded_md()
                .bg(pal.app_bg)
                .overflow_hidden()
                .child(preview),
        )
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .child(asset.display_name.clone()),
        )
        .child(
            div()
                .text_color(pal.text_secondary)
                .child(asset.name.clone()),
        )
        .child(field("分类", asset.profile.clone(), &pal))
        .child(field(
            "模型",
            if asset.model.is_empty() {
                "逻辑对象 / 无独立模型".into()
            } else {
                asset.model.clone()
            },
            &pal,
        ));
    let name = asset.name.clone();
    let favorite = map.asset_browser.favorites.contains(&name);
    let placement_name = name.clone();
    let center_name = name.clone();
    detail
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_1()
                .child(button(
                    if favorite {
                        "★ 已收藏"
                    } else {
                        "☆ 收藏"
                    },
                    favorite,
                    &pal,
                    model,
                    id,
                    move |m| m.asset_browser.toggle_favorite(&name),
                ))
                .child(button(
                    "放置到地图",
                    map.asset_browser.pending_placement.as_deref() == Some(&placement_name),
                    &pal,
                    model,
                    id,
                    move |m| {
                        m.asset_browser.pending_placement = Some(placement_name.clone());
                        Ok(())
                    },
                ))
                .child(button(
                    "添加到视图中心",
                    false,
                    &pal,
                    model,
                    id,
                    move |m| {
                        m.add_asset(&center_name, m.asset_spawn_position())
                            .map(|_| ())
                    },
                )),
        )
        .into_any_element()
}

pub fn render(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
) -> AnyElement {
    let pal = state.theme();
    let browser = &map.asset_browser;
    let entries = filtered_entries(map);
    let count = entries.len();
    let total = map.assets.catalog.entries.len();
    let search_model = model.clone();
    let search_id = id.to_string();
    let resize_model = model.clone();
    let resize_id = id.to_string();
    let height = browser.height;
    let toolbar = div()
        .h(px(34.))
        .flex_shrink_0()
        .px_2()
        .flex()
        .items_center()
        .gap_2()
        .border_b_1()
        .border_color(pal.card_border)
        .child(div().font_weight(FontWeight::BOLD).text_xs().child("资产"))
        .child(
            div()
                .w(px(92.))
                .text_xs()
                .text_color(pal.text_secondary)
                .child(format!("{count} / {total}")),
        )
        .child(
            div()
                .min_w_0()
                .flex_1()
                .px_2()
                .py_1()
                .rounded_sm()
                .bg(pal.app_bg)
                .border_1()
                .border_color(if map.input == Some(MapInput::AssetSearch) {
                    pal.primary_accent
                } else {
                    pal.card_border
                })
                .text_xs()
                .cursor_text()
                .child(if browser.query.is_empty() {
                    div()
                        .text_color(pal.text_secondary)
                        .child("搜索中文名称、Actor ID、模型…")
                        .into_any_element()
                } else {
                    crate::ui::components::render_input_text(
                        &browser.query,
                        map.input == Some(MapInput::AssetSearch),
                        state,
                        model,
                    )
                    .into_any_element()
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    search_model.update(cx, |st, cx| {
                        if st
                            .active_map()
                            .is_some_and(|m| m.input == Some(MapInput::AssetSearch))
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
                            m.input = Some(MapInput::AssetSearch);
                            Ok(())
                        });
                        st.text_input_cursor = st
                            .active_map()
                            .map_or(0, |m| m.asset_browser.query.chars().count());
                        st.text_input_select_anchor = None;
                        cx.notify();
                    });
                }),
        )
        .child(button("清空", false, &pal, model, id, |m| {
            m.asset_browser.query.clear();
            Ok(())
        }))
        .child(button(
            "★ 收藏",
            browser.favorites_only,
            &pal,
            model,
            id,
            |m| {
                m.asset_browser.favorites_only = !m.asset_browser.favorites_only;
                m.asset_browser.category = None;
                Ok(())
            },
        ))
        .child(button(
            if browser.grid {
                "▤ 列表"
            } else {
                "▦ 网格"
            },
            !browser.grid,
            &pal,
            model,
            id,
            |m| {
                m.asset_browser.grid = !m.asset_browser.grid;
                Ok(())
            },
        ))
        .child(button("收起", false, &pal, model, id, |m| {
            m.asset_browser.open = false;
            Ok(())
        }));

    let categories = map.assets.catalog.categories.clone();
    let category_model = model.clone();
    let category_id = id.to_string();
    let selected_category = browser.category.clone();
    let favorites_only = browser.favorites_only;
    let category_panel = div()
        .w(px(162.))
        .flex_shrink_0()
        .min_h_0()
        .flex()
        .flex_col()
        .border_r_1()
        .border_color(pal.card_border)
        .p_1()
        .gap_1()
        .child(button(
            "全部资产",
            browser.category.is_none() && !browser.favorites_only,
            &pal,
            model,
            id,
            |m| {
                m.asset_browser.category = None;
                m.asset_browser.favorites_only = false;
                Ok(())
            },
        ))
        .child(button(
            format!("★ 收藏夹 · {}", browser.favorites.len()),
            browser.favorites_only,
            &pal,
            model,
            id,
            |m| {
                m.asset_browser.category = None;
                m.asset_browser.favorites_only = true;
                Ok(())
            },
        ))
        .child(
            uniform_list(
                "map-asset-categories",
                categories.len(),
                move |range, _, _| {
                    range
                        .map(|i| {
                            let category = &categories[i];
                            let profile = category.profile.clone();
                            div()
                                .h(px(27.))
                                .child(button(
                                    format!("{} · {}", category.label, category.count),
                                    selected_category.as_deref() == Some(&profile)
                                        && !favorites_only,
                                    &pal,
                                    &category_model,
                                    &category_id,
                                    move |m| {
                                        m.asset_browser.category = Some(profile.clone());
                                        m.asset_browser.favorites_only = false;
                                        Ok(())
                                    },
                                ))
                                .into_any_element()
                        })
                        .collect()
                },
            )
            .flex_1()
            .min_h_0(),
        );

    let width = GRID_WIDTH.with(|cache| cache.borrow().get(id).copied().unwrap_or(800.));
    let columns = if browser.grid {
        ((width - 12.) / 108.).floor().max(1.) as usize
    } else {
        1
    };
    let row_count = count.div_ceil(columns);
    let catalog = map.assets.catalog.clone();
    let favorites = browser.favorites.clone();
    let selected = browser.selected.clone();
    let grid = browser.grid;
    let rows_model = model.clone();
    let rows_id = id.to_string();
    let width_id = id.to_string();
    let width_model = model.clone();
    let mut list_hash = std::collections::hash_map::DefaultHasher::new();
    browser.query.hash(&mut list_hash);
    browser.category.hash(&mut list_hash);
    browser.favorites_only.hash(&mut list_hash);
    columns.hash(&mut list_hash);
    let list_id: SharedString = format!("map-asset-results-{}", list_hash.finish()).into();
    let results = div()
        .relative()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            canvas(
                |_, _, _| {},
                move |bounds, _, _, cx| {
                    let actual = f32::from(bounds.size.width);
                    let changed = GRID_WIDTH.with(|cache| {
                        let mut cache = cache.borrow_mut();
                        if cache
                            .get(&width_id)
                            .is_none_or(|previous| (actual - previous).abs() > 1.)
                        {
                            cache.insert(width_id.clone(), actual);
                            true
                        } else {
                            false
                        }
                    });
                    if changed {
                        width_model.update(cx, |_, cx| cx.notify());
                    }
                },
            )
            .absolute()
            .size_full(),
        )
        .child(
            uniform_list(list_id, row_count, move |range, _, _| {
                range
                    .map(|row| {
                        let mut result = div()
                            .h(px(if grid { 100. } else { 37. }))
                            .w_full()
                            .flex()
                            .gap_1()
                            .px_1();
                        for column in 0..columns {
                            let Some(&index) = entries.get(row * columns + column) else {
                                result = result.child(div().flex_1().min_w_0());
                                continue;
                            };
                            let asset = &catalog.entries[index];
                            let name = asset.name.clone();
                            let is_selected = selected.as_deref() == Some(&name);
                            let is_favorite = favorites.contains(&name);
                            let item_model = rows_model.clone();
                            let item_id = rows_id.clone();
                            let drag = AssetDrag {
                                name: name.clone(),
                                tab_id: rows_id.clone(),
                            };
                            let item = div()
                                .id(SharedString::from(format!("asset-item-{name}")))
                                .flex_1()
                                .min_w_0()
                                .h_full()
                                .p_1()
                                .rounded_sm()
                                .border_1()
                                .border_color(if is_selected {
                                    pal.primary_accent
                                } else {
                                    pal.panel_surface
                                })
                                .bg(if is_selected {
                                    pal.card_hover
                                } else {
                                    pal.panel_surface
                                })
                                .hover(|s| s.bg(pal.card_hover))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .gap_1();
                            let item = if grid {
                                item.flex_col()
                                    .child(cube_icon(46.))
                                    .child(
                                        div()
                                            .w_full()
                                            .text_center()
                                            .text_xs()
                                            .text_ellipsis()
                                            .overflow_hidden()
                                            .child(format!(
                                                "{}{}",
                                                if is_favorite { "★ " } else { "" },
                                                asset.display_name
                                            )),
                                    )
                                    .child(
                                        div()
                                            .w_full()
                                            .text_center()
                                            .text_size(px(9.))
                                            .text_color(pal.text_secondary)
                                            .text_ellipsis()
                                            .overflow_hidden()
                                            .child(name.clone()),
                                    )
                            } else {
                                item.child(cube_icon(25.))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_xs()
                                            .text_ellipsis()
                                            .overflow_hidden()
                                            .child(format!(
                                                "{}{}",
                                                if is_favorite { "★ " } else { "" },
                                                asset.display_name
                                            )),
                                    )
                                    .child(
                                        div()
                                            .w(px(235.))
                                            .text_size(px(10.))
                                            .text_color(pal.text_secondary)
                                            .text_ellipsis()
                                            .overflow_hidden()
                                            .child(name.clone()),
                                    )
                            };
                            result = result.child(
                                item.on_mouse_down(MouseButton::Left, move |event, _, cx| {
                                    item_model.update(cx, |st, cx| {
                                        st.clear_active_text_inputs();
                                        if st.active_map().is_some_and(|m| {
                                            m.input.is_some() && m.input_error.is_some()
                                        }) {
                                            return;
                                        }
                                        st.update_map(&item_id, |m| {
                                            m.asset_browser.selected = Some(name.clone());
                                            if event.click_count > 1 {
                                                m.asset_browser.pending_placement =
                                                    Some(name.clone());
                                            }
                                            Ok(())
                                        });
                                        cx.notify();
                                    });
                                })
                                .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone())),
                            );
                        }
                        result.into_any_element()
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
                .child(if map.loading {
                    "正在读取资产目录…"
                } else if total == 0 {
                    "未找到 ActorInfo 资产目录，请在设置中配置游戏资源。"
                } else {
                    "没有匹配的资产"
                })
        }));

    div()
        .h(px(browser.height.clamp(180., 500.)))
        .flex_shrink_0()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(pal.panel_surface)
        .text_color(pal.text_primary)
        .child(
            div()
                .h(px(4.))
                .flex_shrink_0()
                .bg(pal.card_border)
                .cursor_row_resize()
                .hover(|s| s.bg(pal.primary_accent))
                .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    cx.stop_propagation();
                    resize_model.update(cx, |st, cx| {
                        st.update_map(&resize_id, |m| {
                            m.asset_browser.resizing = Some((event.position.y.into(), height));
                            Ok(())
                        });
                        cx.notify();
                    });
                }),
        )
        .child(toolbar)
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .overflow_hidden()
                .child(category_panel)
                .child(results)
                .child(details(map, state, model, id)),
        )
        .into_any_element()
}
