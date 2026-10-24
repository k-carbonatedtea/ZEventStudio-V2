use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::quest_map::types::{get_region_name_for_coord, get_section_for_coord};

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

    let target_marker_id = state
        .quest_map_hovered_marker
        .as_deref()
        .or(state.quest_map_selected_marker.as_deref());
    let cached_markers = state.get_hyrule_markers();
    let fallback_markers;
    let markers_slice: &[crate::ui::views::quest_map::markers_data::HyruleMarker] =
        if cached_markers.is_empty() {
            fallback_markers =
                crate::ui::views::quest_map::markers_data::get_default_hyrule_markers();
            &fallback_markers
        } else {
            cached_markers
        };
    let active_marker = target_marker_id
        .and_then(|mid| markers_slice.iter().find(|m| m.id == mid).cloned());
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
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(div().text_color(pal.text_secondary).child("📍"))
                        .child(
                            div()
                                .text_color(pal.text_secondary)
                                .child(state.t("quests.map_coords")),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .child(coords_label),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(div().text_color(pal.text_secondary).child("🗺️"))
                        .child(
                            div()
                                .text_color(pal.text_secondary)
                                .child(state.t("quests.map_section")),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(pal.primary_accent)
                                .child(section_label),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_color(pal.text_secondary)
                                .child(state.t("quests.map_zoom")),
                        )
                        .child(
                            div()
                                .text_color(pal.text_primary)
                                .child(format!("{}%", zoom_pct)),
                        ),
                )
                .children(active_marker.map(|lm| {
                    let sub_desc = lm
                        .display_sub(is_zh)
                        .map(|s| format!("「{}」", s))
                        .unwrap_or_default();
                    let game_icon = crate::ui::views::quest_map::markers_assets::get_marker_game_icon_path(
                        &lm, true,
                    );
                    let display_name: SharedString = lm.display_name(is_zh).to_string().into();
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(pal.primary_accent.opacity(0.15))
                        .border_1()
                        .border_color(pal.primary_accent.opacity(0.4))
                        .child(if let Some(path) = game_icon {
                            img(path).size(px(15.0)).into_any_element()
                        } else {
                            div().child(lm.icon_char.clone()).into_any_element()
                        })
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .child(display_name),
                        )
                        .children((!sub_desc.is_empty()).then(|| {
                            div().text_color(pal.primary_accent).child(sub_desc)
                        }))
                        .child(div().text_color(pal.text_secondary).child(format!(
                            "({}: {:.0}m)",
                            state.t("quests.map_altitude"),
                            lm.altitude
                        )))
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
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(div().size(px(8.0)).rounded_full().bg(rgb(0xf59e0b)))
                        .child(
                            div()
                                .text_color(pal.text_secondary)
                                .child(state.t("quests.map_legend_main")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(div().size(px(8.0)).rounded_full().bg(rgb(0x06b6d4)))
                        .child(
                            div()
                                .text_color(pal.text_secondary)
                                .child(state.t("quests.map_legend_sub")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(div().size(px(8.0)).rounded_full().bg(rgb(0x10b981)))
                        .child(
                            div()
                                .text_color(pal.text_secondary)
                                .child(state.t("quests.map_legend_mini")),
                        ),
                ),
        )
}
