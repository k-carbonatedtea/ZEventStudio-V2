use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::quest_map::markers_assets::get_marker_game_icon_path;
use crate::ui::views::quest_map::markers_data::{
    get_default_hyrule_markers, get_hyrule_regions, HyruleMarker, HyruleMarkerKind,
};
use crate::ui::views::quest_map::types::world_to_screen;

/// 渲染海拉鲁大区域名称水印文字图层 (底图氛围标识)
pub fn render_region_labels_layer(pan: (f32, f32), zoom: f32, is_zh: bool) -> impl IntoElement {
    let font_size = (16.0 * (zoom / 0.085).clamp(0.7, 1.6)).max(12.0);

    div()
        .absolute()
        .inset_0()
        .children(get_hyrule_regions().iter().filter_map(move |reg| {
            let (sx, sy) = world_to_screen(reg.position.0, reg.position.1, (0.0, 0.0), pan, zoom);
            if !(-100.0..=2200.0).contains(&sx) || !(-60.0..=1400.0).contains(&sy) {
                return None;
            }

            Some(
                div()
                    .absolute()
                    .left(px(sx - 80.0))
                    .top(px(sy - 20.0))
                    .w(px(160.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(font_size))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgba(0xe2e8f038))
                            .whitespace_nowrap()
                            .child(reg.display_name(is_zh)),
                    ),
            )
        }))
}

/// 渲染海拉鲁全境地标标注浮层 (优先使用从项目与游戏路径动态提取并缓存的标点)
pub fn render_markers_layer(
    state: &AppState,
    model: &Entity<AppState>,
    pan: (f32, f32),
    zoom: f32,
    is_zh: bool,
) -> impl IntoElement {
    let show_shrines = state.quest_map_show_shrines;
    let show_stables = state.quest_map_show_stables;
    let show_beasts = state.quest_map_show_beasts;
    let show_towers = state.quest_map_show_towers;
    let show_settlements = state.quest_map_show_settlements;

    let hovered_id = state.quest_map_hovered_marker.as_deref();
    let selected_id = state.quest_map_selected_marker.as_deref();

    // 优先读取内存中的一次性提取地标缓存，若无则使用默认模板
    let cached_markers = state.get_hyrule_markers();
    let fallback_markers;
    let markers: &[HyruleMarker] = if cached_markers.is_empty() {
        fallback_markers = get_default_hyrule_markers();
        &fallback_markers
    } else {
        cached_markers
    };

    div()
        .absolute()
        .inset_0()
        // 1. 大区域水印文字
        .child(render_region_labels_layer(pan, zoom, is_zh))
        // 2. 全部地标标点元素 (带 LOD 与视口裁剪，激活/悬停标点置顶渲染)
        .children({
            let mut list: Vec<_> = markers.iter().filter_map(move |lm| {
                // 类别可见性过滤
                let visible = match lm.kind {
                    HyruleMarkerKind::Shrine => show_shrines,
                    HyruleMarkerKind::Stable => show_stables,
                    HyruleMarkerKind::DivineBeast => show_beasts,
                    HyruleMarkerKind::Tower => show_towers,
                    HyruleMarkerKind::Village | HyruleMarkerKind::Facility | HyruleMarkerKind::Shop => show_settlements,
                };
                if !visible {
                    return None;
                }

                // 视口坐标投影与边界裁剪 (Viewport Culling: 超出屏幕可视范围极速跳过)
                let (sx, sy) = world_to_screen(lm.position.0, lm.position.1, (0.0, 0.0), pan, zoom);
                if !(-60.0..=2200.0).contains(&sx) || !(-60.0..=1400.0).contains(&sy) {
                    return None;
                }

                let is_hovered = hovered_id == Some(lm.id.as_ref());
                let is_selected = selected_id == Some(lm.id.as_ref());

                Some((lm, (sx, sy), is_hovered, is_selected))
            }).collect();

            // 悬停或选中的卡片置于图层最顶层，彻底杜绝被同位置邻近地标遮挡
            list.sort_by_key(|&(_, _, is_hovered, is_selected)| is_hovered || is_selected);

            list.into_iter().map(move |(lm, (sx, sy), is_hovered, is_selected)| {
                render_single_marker_card(
                    lm,
                    (sx, sy),
                    zoom,
                    is_zh,
                    is_hovered,
                    is_selected,
                    model,
                )
            })
        })
}

/// 渲染单个地标标点卡片 (全面接入官方游戏素材贴图与自定义 Mod 标识)
fn render_single_marker_card(
    lm: &HyruleMarker,
    screen_pos: (f32, f32),
    zoom: f32,
    is_zh: bool,
    is_hovered: bool,
    is_selected: bool,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let (sx, sy) = screen_pos;
    let m_click = model.clone();
    let m_hover = model.clone();
    let marker_id = lm.id.clone();
    let marker_pos = lm.position;

    // 主题边框与文字高亮色
    let (border_color, text_color) = match lm.kind {
        HyruleMarkerKind::Shrine => (
            if is_selected { rgb(0xffffff) } else if is_hovered { rgb(0x38bdf8) } else { rgb(0x0284c7) },
            rgb(0x38bdf8),
        ),
        HyruleMarkerKind::Tower => (
            if is_selected { rgb(0xffffff) } else if is_hovered { rgb(0x67e8f9) } else { rgb(0x06b6d4) },
            rgb(0x67e8f9),
        ),
        HyruleMarkerKind::Stable => (
            if is_selected { rgb(0xffffff) } else if is_hovered { rgb(0xfde047) } else { rgb(0xf59e0b) },
            rgb(0xfde047),
        ),
        HyruleMarkerKind::DivineBeast => (
            if is_selected { rgb(0xffffff) } else { rgb(0x38bdf8) },
            rgb(0x7dd3fc),
        ),
        HyruleMarkerKind::Village => (
            if is_selected { rgb(0xffffff) } else if is_hovered { rgb(0x5eead4) } else { rgb(0x14b8a6) },
            rgb(0x5eead4),
        ),
        HyruleMarkerKind::Shop => (
            if is_selected { rgb(0xffffff) } else if is_hovered { rgb(0xfbbf24) } else { rgb(0xd97706) },
            rgb(0xfef3c7),
        ),
        HyruleMarkerKind::Facility => (
            if is_selected { rgb(0xffffff) } else if is_hovered { rgb(0xf43f5e) } else { rgb(0xe11d48) },
            rgb(0xfecdd3),
        ),
    };

    // 缩放 LOD 策略：店铺密集聚集于村落中，中等缩放下以精致徽章图标展示，悬停/选中或深度放大时展开详细名称
    let show_full_label = is_hovered
        || is_selected
        || lm.kind == HyruleMarkerKind::DivineBeast
        || (lm.kind != HyruleMarkerKind::Shop && zoom >= 0.12)
        || (lm.kind == HyruleMarkerKind::Shop && zoom >= 0.28);
    let is_compact = lm.kind == HyruleMarkerKind::Shrine && zoom < 0.06 && !is_hovered && !is_selected;

    // 获取官方游戏贴图路径 (全品类覆盖)
    let icon_path_opt = get_marker_game_icon_path(lm, is_hovered || is_selected);

    // 动态像素尺寸计算
    let icon_px = match lm.kind {
        HyruleMarkerKind::DivineBeast => if is_selected || is_hovered { 36.0 } else { 30.0 },
        HyruleMarkerKind::Tower => if is_selected || is_hovered { 30.0 } else { 24.0 },
        HyruleMarkerKind::Stable => if is_selected || is_hovered { 28.0 } else { 22.0 },
        HyruleMarkerKind::Shrine => {
            if is_selected || is_hovered {
                28.0
            } else if is_compact {
                14.0
            } else if zoom >= 0.12 {
                24.0
            } else {
                20.0
            }
        }
        HyruleMarkerKind::Shop => if is_selected || is_hovered { 26.0 } else { 20.0 },
        HyruleMarkerKind::Facility => if is_selected || is_hovered { 30.0 } else { 24.0 },
        HyruleMarkerKind::Village => if is_selected || is_hovered { 28.0 } else { 22.0 },
    };

    let half_size = icon_px / 2.0;
    // 使用对称的固定定位盒 (260px)，确保当药丸标签展开或收起时，图标中心永远绝对锁定在 (sx, sy) 零漂移
    let anchor_w = 260.0;

    let mid_click = marker_id.to_string();
    let mid_hover = marker_id.to_string();
    let icon_char = lm.icon_char.clone();

    div()
        .id(SharedString::from(format!("map_marker_{}", marker_id)))
        .absolute()
        .left(px(sx - anchor_w / 2.0))
        .top(px(sy - half_size))
        .w(px(anchor_w))
        .cursor_pointer()
        .flex()
        .flex_col()
        .items_center()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            let mid = mid_click.clone();
            let pos = marker_pos;
            m_click.update(cx, |st, cx| {
                st.quest_map_selected_marker = Some(mid);
                st.focus_landmark_on_map(pos);
                cx.notify();
            });
        })
        .on_hover(move |hovered, _, cx| {
            let mid = mid_hover.clone();
            m_hover.update(cx, |st, cx| {
                st.quest_map_hovered_marker = if *hovered { Some(mid) } else { None };
                cx.notify();
            });
        })
        // 图标主体：全品类优先使用官方游戏素材贴图
        .child(
            if let Some(path) = icon_path_opt {
                div()
                    .size(px(icon_px))
                    .flex()
                    .items_center()
                    .justify_center()
                    .children((is_selected || is_hovered).then(|| {
                        div()
                            .absolute()
                            .inset_0()
                            .rounded_full()
                            .bg(border_color.opacity(0.35))
                    }))
                    .children((lm.is_custom && !is_selected && !is_hovered).then(|| {
                        // 自定义 Mod 标点在全景下附带一层精致金圈，不弹卡片、不挡视线、零排版开销
                        div()
                            .absolute()
                            .inset_0()
                            .rounded_full()
                            .border_1()
                            .border_color(rgb(0xeab308).opacity(0.85))
                    }))
                    .child(
                        img(path)
                            .size(px(icon_px))
                    )
                    .into_any_element()
            } else {
                // 兜底拟态徽章
                div()
                    .size(px(icon_px))
                    .rounded_md()
                    .bg(rgba(0x0f172aee))
                    .border_2()
                    .border_color(if lm.is_custom { rgb(0xeab308) } else { border_color })
                    .shadow_md()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(if is_compact { px(0.0) } else { px(12.0) })
                    .text_color(rgb(0xffffff))
                    .child(if is_compact { SharedString::default() } else { icon_char })
                    .into_any_element()
            }
        )
        // 名称药丸标签 (当放大、悬停或属于自定义 Mod 标点时呈现)
        .children(show_full_label.then(|| {
            div()
                .mt_0p5()
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .bg(rgba(0x0f172aee))
                .border_1()
                .border_color(if lm.is_custom { rgb(0xeab308) } else { border_color.opacity(0.8) })
                .shadow_sm()
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if lm.is_custom { rgb(0xfde047) } else { text_color })
                                .whitespace_nowrap()
                                .child(lm.display_name(is_zh).to_string()),
                        )
                        .children(lm.is_custom.then(|| {
                            div()
                                .px_1()
                                .rounded_xs()
                                .bg(rgb(0xeab308))
                                .text_color(rgb(0x0f172a))
                                .text_size(px(8.0))
                                .font_weight(FontWeight::BOLD)
                                .child("MOD")
                        }))
                )
                // 副标题 (如神庙试炼名 / 神兽全称 / 自定义标点说明)
                .children(lm.display_sub(is_zh).map(|sub| {
                    div()
                        .text_size(px(8.5))
                        .text_color(if lm.is_custom { rgb(0xfef08a) } else { rgb(0x94a3b8) })
                        .whitespace_nowrap()
                        .child(format!("- {} -", sub))
                }))
        }))
}
