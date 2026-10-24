pub mod map_asset_indicator;
pub mod map_bottom_hud;
pub mod map_canvas;
pub mod map_layer_filters;
pub mod map_loading_overlay;
pub mod map_renderer;
pub mod map_toolbar;
pub mod markers_assets;
pub mod markers_data;
pub mod markers_extractor;
pub mod markers_landmarks;
pub mod markers_layer;
pub mod markers_shrines;
pub mod pin_tooltip;
pub mod pins_layer;
pub mod tiles_layer;
pub mod types;

use gpui::*;
use crate::ui::state::AppState;
use map_bottom_hud::render_map_bottom_hud;
use map_canvas::render_map_canvas;
use map_loading_overlay::render_map_loading_overlay;
use map_toolbar::render_map_header_hud;
use pin_tooltip::render_pin_tooltip_layer;
use pins_layer::collect_rendered_pins;

/// 渲染 2D 海拉鲁任务制作与标点追踪全景地图工作区视图
pub fn render_quest_map_view(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let pins = collect_rendered_pins(state, (0.0, 0.0), state.quest_map_pan, state.quest_map_zoom);

    div()
        .id("quest_map_view_root")
        .size_full()
        .flex()
        .flex_col()
        .bg(pal.app_bg)
        .overflow_hidden()
        // 1. 顶部 HUD 工具栏
        .child(render_map_header_hud(state, model))
        // 2. 中间 2D 交互地图与浮层
        .child(
            div()
                .flex_1()
                .size_full()
                .relative()
                .overflow_hidden()
                .child(render_map_canvas(state, model))
                .child(render_pin_tooltip_layer(&pins, state, model))
                .children(state.quest_map_is_extracting_markers.then(|| {
                    render_map_loading_overlay(state)
                })),
        )
        // 3. 底部世界坐标与图例 HUD 状态栏
        .child(render_map_bottom_hud(state))
}
