use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::quest_map::map_layer_filters::render_map_layer_filters;
use crate::ui::views::quest_map::markers_data::get_all_hyrule_markers;
use crate::ui::views::quest_map::types::{get_region_name_for_coord, get_section_for_coord};

/// 渲染 2D 地图顶部 HUD 工具栏 (任务快速切换、阶段直达、视口与图层控制)
pub fn render_map_header_hud(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let selected_quest = state.get_selected_quest();

    let model_grid = model.clone();
    let model_all_pins = model.clone();
    let model_center = model.clone();
    let model_zoom_in = model.clone();
    let model_zoom_out = model.clone();
    let model_reset = model.clone();

    let show_grid = state.quest_map_show_grid;
    let show_all = state.quest_map_show_all_pins;
    let is_extracting = state.quest_map_is_extracting_tiles;
    let tiles_count = crate::ui::views::quest_map::tiles_layer::count_cached_map_tiles(state.project_path.as_deref());
    let model_extract = model.clone();

    div()
        .h(px(42.0))
        .w_full()
        .px_3()
        .bg(pal.panel_surface.opacity(0.92))
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        // 左侧：选中任务信息与阶段直达 Chips
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .children(selected_quest.map(|q| {
                    let q_title = q.display_title.as_deref().unwrap_or(&q.name);
                    let (badge_bg, badge_fg) = match q.quest_type {
                        crate::model::quest::QuestType::Main => (pal.secondary_accent.opacity(0.2), pal.secondary_accent),
                        crate::model::quest::QuestType::Sub => (pal.primary_accent.opacity(0.2), pal.primary_accent),
                        crate::model::quest::QuestType::Mini => (pal.primary_accent_hover.opacity(0.2), pal.primary_accent_hover),
                    };

                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(badge_bg)
                                        .text_color(badge_fg)
                                        .child(state.t(q.quest_type.label_key())),
                                )
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_primary)
                                        .child(q_title.to_string()),
                                ),
                        )
                        // 阶段标点快速直达
                        .children(q.steps.iter().enumerate().filter(|(_, s)| !s.indicators.is_empty()).map(|(s_idx, s)| {
                            let m_step = model.clone();
                            let pos = s.indicators[0].position;
                            div()
                                .cursor_pointer()
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .bg(pal.card_hover)
                                .text_size(px(10.5))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.primary_accent.opacity(0.25)).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m_step.update(cx, |st, cx| {
                                        st.focus_pin_on_map(pos);
                                        cx.notify();
                                    });
                                })
                                .child(format!("📍 Step {}: {}", s_idx + 1, s.name))
                        }))
                }))
                .children(selected_quest.is_none().then(|| {
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .text_size(px(12.5))
                        .text_color(pal.text_secondary)
                        .child("🗺️")
                        .child(state.t("quests.map_title"))
                })),
        )
        // 右侧：视口工具与图层开关
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                // 地图标注分类图层过滤器 (神庙/驿站/神兽/塔/城镇)
                .child(render_map_layer_filters(state, model))
                // 网格开关
                .child(
                    div()
                        .id("btn_toggle_map_grid")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(if show_grid { pal.primary_accent.opacity(0.2) } else { pal.card_hover })
                        .border_1()
                        .border_color(if show_grid { pal.primary_accent } else { pal.panel_border })
                        .text_size(px(11.0))
                        .text_color(if show_grid { pal.primary_accent } else { pal.text_secondary })
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_grid.update(cx, |st, cx| {
                                st.quest_map_show_grid = !st.quest_map_show_grid;
                                cx.notify();
                            });
                        })
                        .child(state.t("quests.map_toggle_grid")),
                )
                // 瓦片缓存数量指示
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(pal.card_hover)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(11.0))
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(div().child("🗺️"))
                        .child(div().text_color(pal.text_secondary).child(state.t("quests.map_tiles")))
                        .child(div().font_weight(FontWeight::BOLD).text_color(pal.text_primary).child(format!("{}/129", tiles_count))),
                )
                // 提取瓦片按钮 (优先从打开的项目提取)
                .child(
                    div()
                        .id("btn_extract_map_tiles")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(if is_extracting { pal.secondary_accent.opacity(0.3) } else { pal.card_hover })
                        .border_1()
                        .border_color(if is_extracting { pal.secondary_accent } else { pal.panel_border })
                        .text_size(px(11.0))
                        .text_color(if is_extracting { pal.secondary_accent } else { pal.text_primary })
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.25)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            if !is_extracting {
                                let m = model_extract.clone();
                                model_extract.update(cx, |st, cx| {
                                    st.trigger_extract_map_tiles(&m, cx);
                                });
                            }
                        })
                        .child(if is_extracting {
                            state.t("quests.extracting_tiles")
                        } else {
                            state.t("quests.extract_tiles")
                        }),
                )
                // 显示全部标点 / 仅当前任务
                .child(
                    div()
                        .id("btn_toggle_all_pins")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(if show_all { pal.secondary_accent.opacity(0.2) } else { pal.card_hover })
                        .border_1()
                        .border_color(if show_all { pal.secondary_accent } else { pal.panel_border })
                        .text_size(px(11.0))
                        .text_color(if show_all { pal.secondary_accent } else { pal.text_secondary })
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_all_pins.update(cx, |st, cx| {
                                st.quest_map_show_all_pins = !st.quest_map_show_all_pins;
                                cx.notify();
                            });
                        })
                        .child(if show_all { state.t("quests.map_all_pins") } else { state.t("quests.map_selected_pins") }),
                )
                // 居中聚焦任务
                .child(
                    div()
                        .id("btn_map_center_quest")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(pal.card_hover)
                        .text_size(px(11.0))
                        .text_color(pal.text_primary)
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.25)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_center.update(cx, |st, cx| {
                                st.focus_selected_quest_on_map();
                                cx.notify();
                            });
                        })
                        .child(state.t("quests.map_center_quest")),
                )
                // 放大
                .child(
                    div()
                        .cursor_pointer()
                        .size(px(24.0))
                        .rounded_sm()
                        .bg(pal.card_hover)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.25)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_zoom_in.update(cx, |st, cx| {
                                st.zoom_quest_map(1.25);
                                cx.notify();
                            });
                        })
                        .child("➕"),
                )
                // 缩小
                .child(
                    div()
                        .cursor_pointer()
                        .size(px(24.0))
                        .rounded_sm()
                        .bg(pal.card_hover)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.25)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_zoom_out.update(cx, |st, cx| {
                                st.zoom_quest_map(0.80);
                                cx.notify();
                            });
                        })
                        .child("➖"),
                )
                // 重置全图
                .child(
                    div()
                        .cursor_pointer()
                        .size(px(24.0))
                        .rounded_sm()
                        .bg(pal.card_hover)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.25)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_reset.update(cx, |st, cx| {
                                st.reset_quest_map_view();
                                cx.notify();
                            });
                        })
                        .child("⟲"),
                ),
        )
}

/// 渲染 2D 地图底部状态栏 HUD (实时世界坐标、分区指示器、缩放比率与图例)
pub fn render_map_bottom_hud(state: &AppState) -> impl IntoElement {
    let pal = state.theme();
    let is_zh = state.lang == crate::i18n::Language::Zh;

    let (coords_label, section_label) = if let Some((wx, wz)) = state.quest_map_hovered_coord {
        let sec = get_section_for_coord(wx, wz);
        let reg = get_region_name_for_coord(wx, wz, is_zh);
        (
            format!("X: {:.0}, Z: {:.0}", wx, wz),
            format!("{} • {}", sec, reg),
        )
    } else {
        ("X: 0, Z: 0".to_string(), "E-4 • Central Hyrule".to_string())
    };

    let target_marker_id = state.quest_map_hovered_marker.as_deref()
        .or(state.quest_map_selected_marker.as_deref());
    let active_marker = target_marker_id.and_then(|mid| get_all_hyrule_markers().find(|m| m.id == mid));
    let zoom_pct = (state.quest_map_zoom / 0.085 * 100.0).round() as i32;

    div()
        .h(px(28.0))
        .w_full()
        .px_3()
        .bg(pal.panel_surface.opacity(0.92))
        .border_t_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(11.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div().flex().items_center().gap_1()
                        .child(div().text_color(pal.text_secondary).child("📍"))
                        .child(div().text_color(pal.text_secondary).child(state.t("quests.map_coords")))
                        .child(div().font_weight(FontWeight::BOLD).text_color(pal.text_primary).child(coords_label)),
                )
                .child(
                    div().flex().items_center().gap_1()
                        .child(div().text_color(pal.text_secondary).child("🗺️"))
                        .child(div().text_color(pal.text_secondary).child(state.t("quests.map_section")))
                        .child(div().font_weight(FontWeight::MEDIUM).text_color(pal.primary_accent).child(section_label)),
                )
                .child(
                    div().flex().items_center().gap_1()
                        .child(div().text_color(pal.text_secondary).child(state.t("quests.map_zoom")))
                        .child(div().text_color(pal.text_primary).child(format!("{}%", zoom_pct))),
                )
                .children(active_marker.map(|lm| {
                    let sub_desc = lm.display_sub(is_zh).map(|s| format!("「{}」", s)).unwrap_or_default();
                    let game_icon = crate::ui::views::quest_map::markers_assets::get_marker_game_icon_path(lm, true);
                    div()
                        .flex().items_center().gap_1p5().px_2().py_0p5().rounded_sm()
                        .bg(pal.primary_accent.opacity(0.15)).border_1().border_color(pal.primary_accent.opacity(0.4))
                        .child(if let Some(path) = game_icon {
                            img(path).size(px(15.0)).into_any_element()
                        } else {
                            div().child(lm.icon_char).into_any_element()
                        })
                        .child(div().font_weight(FontWeight::BOLD).text_color(pal.text_primary).child(lm.display_name(is_zh)))
                        .children((!sub_desc.is_empty()).then(|| div().text_color(pal.primary_accent).child(sub_desc)))
                        .child(div().text_color(pal.text_secondary).child(format!("({}: {:.0}m)", state.t("quests.map_altitude"), lm.altitude)))
                })),
        )
        // 右侧：图例指示
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .text_size(px(10.5))
                .child(
                    div().flex().items_center().gap_1()
                        .child(div().size(px(8.0)).rounded_full().bg(rgb(0xf59e0b)))
                        .child(div().text_color(pal.text_secondary).child(state.t("quests.map_legend_main"))),
                )
                .child(
                    div().flex().items_center().gap_1()
                        .child(div().size(px(8.0)).rounded_full().bg(rgb(0x06b6d4)))
                        .child(div().text_color(pal.text_secondary).child(state.t("quests.map_legend_sub"))),
                )
                .child(
                    div().flex().items_center().gap_1()
                        .child(div().size(px(8.0)).rounded_full().bg(rgb(0x10b981)))
                        .child(div().text_color(pal.text_secondary).child(state.t("quests.map_legend_mini"))),
                ),
        )
}
