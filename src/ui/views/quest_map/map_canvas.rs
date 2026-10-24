use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::quest_map::map_renderer::{draw_hyrule_base_map, draw_section_grid};
use crate::ui::views::quest_map::pins_layer::{
    collect_rendered_pins, draw_quest_trajectories_coords, extract_quest_trajectories,
    render_pins_elements,
};
use crate::ui::views::quest_map::types::{
    screen_to_world, world_to_screen, HYRULE_LANDMARKS,
};

/// 渲染 2D 地图核心交互画布容器 (支持鼠标平移拖拽、滚轮缩放、标点交互)
pub fn render_map_canvas(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pan = state.quest_map_pan;
    let zoom = state.quest_map_zoom;
    let show_grid = state.quest_map_show_grid;
    let hovered_coord = state.quest_map_hovered_coord;
    let is_zh = state.lang == crate::i18n::Language::Zh;

    let is_dragging = state.quest_map_is_dragging;

    let model_down = model.clone();
    let model_up = model.clone();
    let model_mid_down = model.clone();
    let model_mid_up = model.clone();
    let model_move = model.clone();
    let model_scroll = model.clone();

    // 预收集渲染标点实体 (DOM 相对坐标系)
    let pins = collect_rendered_pins(state, (0.0, 0.0), pan, zoom);

    // 预提取任务阶段推进连线世界坐标段
    let trajectories = extract_quest_trajectories(state);

    div()
        .id("quest_2d_map_canvas")
        .relative()
        .size_full()
        .bg(rgb(0x0a0f1d)) // 深空黑夜海拉鲁外海基底
        .overflow_hidden()
        .cursor(if is_dragging {
            CursorStyle::ClosedHand
        } else {
            CursorStyle::Arrow
        })
        // 鼠标左键按下：开始拖拽平移
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_down.update(cx, |st, cx| {
                st.quest_map_is_dragging = true;
                st.quest_map_drag_start = (mx, my);
                cx.notify();
            });
        })
        // 鼠标左键抬起：停止拖拽
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            model_up.update(cx, |st, cx| {
                st.quest_map_is_dragging = false;
                cx.notify();
            });
        })
        // 鼠标中键按下：直接开始拖拽平移
        .on_mouse_down(MouseButton::Middle, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_mid_down.update(cx, |st, cx| {
                st.quest_map_is_dragging = true;
                st.quest_map_drag_start = (mx, my);
                cx.notify();
            });
        })
        // 鼠标中键抬起：停止拖拽
        .on_mouse_up(MouseButton::Middle, move |_, _, cx| {
            model_mid_up.update(cx, |st, cx| {
                st.quest_map_is_dragging = false;
                cx.notify();
            });
        })
        // 鼠标移动：计算世界坐标并平移画布
        .on_mouse_move(move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();

            model_move.update(cx, |st, cx| {
                if st.quest_map_is_dragging {
                    let dx = mx - st.quest_map_drag_start.0;
                    let dy = my - st.quest_map_drag_start.1;
                    st.quest_map_pan.0 += dx;
                    st.quest_map_pan.1 += dy;
                    st.quest_map_drag_start = (mx, my);
                }

                let sidebar_w = if st.sidebar_open { 48.0 + st.sidebar_width + 4.0 } else { 48.0 };
                let top_bar_h = 156.0;
                let local_mx = mx - sidebar_w;
                let local_my = my - top_bar_h;
                let (wx, wz) = screen_to_world(local_mx, local_my, (0.0, 0.0), st.quest_map_pan, st.quest_map_zoom);
                st.quest_map_hovered_coord = Some((wx, wz));

                cx.notify();
            });
        })
        // 滚轮缩放：以鼠标当前位置为锚点缩放
        .on_scroll(move |e, _, cx| {
            let dy: f32 = match e.delta {
                ScrollDelta::Pixels(pt) => pt.y.into(),
                ScrollDelta::Lines(pt) => pt.y * 20.0,
            };
            let factor = if dy > 0.0 { 1.15 } else { 0.85 };

            model_scroll.update(cx, |st, cx| {
                let old_zoom = st.quest_map_zoom;
                let new_zoom = (old_zoom * factor).clamp(0.015, 2.5);

                let mx: f32 = e.position.x.into();
                let my: f32 = e.position.y.into();
                let sidebar_w = if st.sidebar_open { 48.0 + st.sidebar_width + 4.0 } else { 48.0 };
                let top_bar_h = 156.0;
                let local_mx = mx - sidebar_w;
                let local_my = my - top_bar_h;

                // 保持鼠标指向的世界坐标在缩放前后不变
                st.quest_map_pan.0 = local_mx - (local_mx - st.quest_map_pan.0) * (new_zoom / old_zoom);
                st.quest_map_pan.1 = local_my - (local_my - st.quest_map_pan.1) * (new_zoom / old_zoom);
                st.quest_map_zoom = new_zoom;

                cx.notify();
            });
        })
        // 1. 底层降级底图与地形轮廓 (当瓦片未覆盖或加载时作为后备底图)
        .child(
            div()
                .absolute()
                .inset_0()
                .child(
                    canvas(
                        |_bounds, _window, _cx| {},
                        move |bounds, _, window, _cx| {
                            let orig_x: f32 = bounds.origin.x.into();
                            let orig_y: f32 = bounds.origin.y.into();
                            let center = (orig_x, orig_y);

                            // 绘制海拉鲁主体轮廓与特征水体 (作为后备底图)
                            draw_hyrule_base_map(bounds, center, pan, zoom, window);
                        },
                    )
                    .size_full(),
                ),
        )
        // 2. 真实海拉鲁高精卫星地图瓦片图层 (无缝微重叠消除接缝)
        .child(crate::ui::views::quest_map::tiles_layer::render_tiles_layer(pan, zoom, state.project_path.as_deref()))
        // 3. 顶层分区网格与任务推进轨迹连线 (覆盖在瓦片之上，开启时清晰呈现，关闭时不遮挡)
        .child(
            div()
                .absolute()
                .inset_0()
                .child(
                    canvas(
                        |_bounds, _window, _cx| {},
                        move |bounds, _, window, _cx| {
                            let orig_x: f32 = bounds.origin.x.into();
                            let orig_y: f32 = bounds.origin.y.into();
                            let center = (orig_x, orig_y);

                            // 当用户在工具栏开启网格时，在瓦片上方绘制清晰的分区网格
                            if show_grid {
                                draw_section_grid(center, pan, zoom, hovered_coord, window);
                            }

                            // 绘制任务标点间推进连线轨迹 (位于瓦片之上)
                            draw_quest_trajectories_coords(&trajectories, center, pan, zoom, window);
                        },
                    )
                    .size_full(),
                ),
        )
        // 4. 地理地标与塔微标识浮层
        .child(render_landmarks_overlay(pan, zoom, is_zh))
        // 5. 任务标点浮层 (可点击交互)
        .child(render_pins_elements(&pins, model))
}

/// 渲染主要城镇与希卡之塔常驻浮层微标签
fn render_landmarks_overlay(pan: (f32, f32), zoom: f32, is_zh: bool) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .children(HYRULE_LANDMARKS.iter().map(move |lm| {
            let (sx, sy) = world_to_screen(lm.position.0, lm.position.1, (0.0, 0.0), pan, zoom);
            let name = if is_zh { lm.name_zh } else { lm.name_en };
            let (bg, border) = if lm.is_tower {
                (rgba(0x06b6d430), rgba(0x06b6d480))
            } else {
                (rgba(0x0f172acc), rgba(0x38bdf840))
            };

            div()
                .absolute()
                .left(px(sx - 20.0))
                .top(px(sy - 12.0))
                .flex()
                .items_center()
                .gap_1()
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .bg(bg)
                .border_1()
                .border_color(border)
                .text_size(px(10.0))
                .text_color(if lm.is_tower { rgb(0x38bdf8) } else { rgb(0xe2e8f0) })
                .whitespace_nowrap()
                .child(lm.icon)
                .child(name)
        }))
}
