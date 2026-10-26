use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::quest_map::map_renderer::{draw_hyrule_base_map, draw_section_grid};
use crate::ui::views::quest_map::pins_layer::{
    collect_rendered_pins, draw_quest_trajectories_coords, extract_quest_trajectories,
    render_pins_elements,
};
use crate::ui::views::quest_map::types::{
    screen_to_world,
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
        // 鼠标左键按下：开始拖拽平移 (当有模态框开启时忽略)
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_down.update(cx, |st, cx| {
                if st.has_active_modal() {
                    return;
                }
                st.quest_map_is_dragging = true;
                st.quest_map_drag_start = (mx, my);
                st.quest_map_drag_dist = 0.0;
                cx.notify();
            });
        })
        // 鼠标左键抬起：停止拖拽；若未产生有效平移，则判定为左键点击地图空白处
        .on_mouse_up(MouseButton::Left, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_up.update(cx, |st, cx| {
                if st.has_active_modal() {
                    st.quest_map_is_dragging = false;
                    return;
                }
                st.quest_map_is_dragging = false;
                // 位移低于 6 像素判定为左键点击空白处
                if st.quest_map_drag_dist < 6.0 {
                    let (ox, oy) = st.get_map_canvas_origin();
                    let local_mx = mx - ox;
                    let local_my = my - oy;
                    let (wx, wz) = screen_to_world(local_mx, local_my, (0.0, 0.0), st.quest_map_pan, st.quest_map_zoom);

                    // 1. 若此前有地图素材处于发光状态，清空发光
                    st.quest_map_selected_marker = None;
                    // 2. 将单一标点移动到新点击位置 (保持单一标点)
                    st.quest_map_user_pin = Some((wx, wz));
                }
                st.quest_map_drag_dist = 0.0;
                cx.notify();
            });
        })
        // 鼠标中键按下：直接开始拖拽平移
        .on_mouse_down(MouseButton::Middle, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_mid_down.update(cx, |st, cx| {
                if st.has_active_modal() {
                    return;
                }
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
                if st.has_active_modal() {
                    return;
                }
                if st.quest_map_is_dragging {
                    let dx = mx - st.quest_map_drag_start.0;
                    let dy = my - st.quest_map_drag_start.1;
                    st.quest_map_drag_dist += dx.abs() + dy.abs();
                    st.quest_map_pan.0 += dx;
                    st.quest_map_pan.1 += dy;
                    st.quest_map_drag_start = (mx, my);
                }

                let (ox, oy) = st.get_map_canvas_origin();
                let local_mx = mx - ox;
                let local_my = my - oy;
                let (wx, wz) = screen_to_world(local_mx, local_my, (0.0, 0.0), st.quest_map_pan, st.quest_map_zoom);
                st.quest_map_hovered_coord = Some((wx, wz));

                cx.notify();
            });
        })
        // 滚轮缩放：以鼠标当前位置为锚点缩放 (当模态框激活时不缩放背景地图)
        .on_scroll(move |e, _, cx| {
            let dy: f32 = match e.delta {
                ScrollDelta::Pixels(pt) => pt.y.into(),
                ScrollDelta::Lines(pt) => pt.y * 20.0,
            };
            let factor = if dy > 0.0 { 1.15 } else { 0.85 };

            model_scroll.update(cx, |st, cx| {
                if st.has_active_modal() {
                    return;
                }
                let old_zoom = st.quest_map_zoom;
                let new_zoom = (old_zoom * factor).clamp(0.015, 2.5);

                let mx: f32 = e.position.x.into();
                let my: f32 = e.position.y.into();
                let (ox, oy) = st.get_map_canvas_origin();
                let local_mx = mx - ox;
                let local_my = my - oy;

                // 保持鼠标指向的世界坐标在缩放前后不变
                st.quest_map_pan.0 = local_mx - (local_mx - st.quest_map_pan.0) * (new_zoom / old_zoom);
                st.quest_map_pan.1 = local_my - (local_my - st.quest_map_pan.1) * (new_zoom / old_zoom);
                st.quest_map_zoom = new_zoom;

                cx.notify();
            });
        })
        // 1. 纯代码绘制海拉鲁矢量极速底图 (当开启矢量极速模式，或卫星瓦片不存在时呈现，毫秒级满帧渲染)
        .children((state.quest_map_use_vector_mode || !crate::ui::views::quest_map::tiles_layer::get_unified_world_map_path(state.project_path.as_deref()).is_some()).then(|| {
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

                            // 绘制经典纯代码海拉鲁主体轮廓与特征水体
                            draw_hyrule_base_map(bounds, center, pan, zoom, window);
                        },
                    )
                    .size_full(),
                )
        }))
        // 2. 真实海拉鲁高精卫星地图瓦片图层 (仅在卫星实景模式下挂载)
        .children((!state.quest_map_use_vector_mode).then(|| {
            crate::ui::views::quest_map::tiles_layer::render_tiles_layer(pan, zoom, state.project_path.as_deref())
        }))
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
        // 4. 海拉鲁全境地标浮层 (包含神庙、驿站、四神兽、希卡塔、主要城镇与大区域水印)
        .child(crate::ui::views::quest_map::markers_layer::render_markers_layer(state, model, pan, zoom, is_zh))
        // 5. 任务标点浮层 (可点击交互)
        .child(render_pins_elements(&pins, model))
        // 6. 用户单点交互标点图层 (全局单一标点，支持点击自身重置清空)
        .child(crate::ui::views::quest_map::user_pin_layer::render_user_pin_layer(state, model, pan, zoom))
}
