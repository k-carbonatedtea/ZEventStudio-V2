use gpui::*;
use crate::ui::canvas::edges::render_edges_layer;
use crate::ui::canvas::grid::draw_canvas_grid;
use crate::ui::canvas::groups::render_groups_layer;
use crate::ui::canvas::nodes::render_nodes_layer;
use crate::ui::quick_add::render_quick_add_modal;
use crate::ui::state::{AppState, DragState};

pub fn render_flow_canvas(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pan = state.pan;
    let zoom = state.zoom;
    let drag_state = state.drag_state.clone();
    let quick_add = state.quick_add_modal.clone();

    let model_mouse_down = model.clone();
    let model_mouse_move = model.clone();
    let model_mouse_up = model.clone();
    let model_scroll = model.clone();

    div()
        .id("flow_canvas")
        .relative()
        .flex_1()
        .size_full()
        .overflow_hidden()
        .bg(hsla(240.0 / 360.0, 0.06, 0.09, 1.0))
        .cursor(match &drag_state {
            DragState::PanCanvas { .. } => CursorStyle::ClosedHand,
            DragState::Connecting { .. } => CursorStyle::Crosshair,
            _ => CursorStyle::Arrow,
        })
        // Mouse down on empty canvas (Ctrl + Left Click 或中键拖动画布)
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            let is_ctrl = e.modifiers.control;
            model_mouse_down.update(cx, |st, cx| {
                // 如果子组件（如引脚已激活 Connecting，或节点标头已激活 DragNodes），则不重置状态
                if matches!(st.drag_state, DragState::Connecting { .. } | DragState::DragNodes { .. }) {
                    return;
                }
                st.selected_node_id = None;
                st.quick_add_modal = None;
                if is_ctrl {
                    st.drag_state = DragState::PanCanvas {
                        start_mouse: (mx, my),
                        start_pan: st.pan,
                    };
                } else {
                    st.drag_state = DragState::None;
                }
                cx.notify();
            });
        })

        // 中键鼠标按下直接平移画布
        .on_mouse_down(MouseButton::Middle, {
            let model_middle = model.clone();
            move |e, _, cx| {
                let mx: f32 = e.position.x.into();
                let my: f32 = e.position.y.into();
                model_middle.update(cx, |st, cx| {
                    st.selected_node_id = None;
                    st.quick_add_modal = None;
                    st.drag_state = DragState::PanCanvas {
                        start_mouse: (mx, my),
                        start_pan: st.pan,
                    };
                    cx.notify();
                });
            }
        })
        .on_mouse_up(MouseButton::Middle, {
            let model_mid_up = model.clone();
            move |_, _, cx| {
                model_mid_up.update(cx, |st, cx| {
                    if matches!(st.drag_state, DragState::PanCanvas { .. }) {
                        st.drag_state = DragState::None;
                        cx.notify();
                    }
                });
            }
        })
        // Right click to open Quick Add modal on canvas, or finish right-click node drag
        .on_mouse_down(MouseButton::Right, {
            let model_right = model.clone();
            move |e, _, cx| {
                let mx: f32 = e.position.x.into();
                let my: f32 = e.position.y.into();
                model_right.update(cx, |st, cx| {
                    if matches!(st.drag_state, DragState::DragNodes { .. }) {
                        return;
                    }
                    let (canvas_x, canvas_y) = st.screen_to_canvas(mx, my);
                    st.quick_add_modal = Some((canvas_x, canvas_y, None));
                    cx.notify();
                });
            }
        })
        .on_mouse_up(MouseButton::Right, {
            let model_right_up = model.clone();
            move |_, _, cx| {
                model_right_up.update(cx, |st, cx| {
                    if matches!(st.drag_state, DragState::DragNodes { .. }) {
                        st.drag_state = DragState::None;
                        cx.notify();
                    }
                });
            }
        })


        // Mouse move
        .on_mouse_move(move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_mouse_move.update(cx, |st, cx| {
                match &st.drag_state.clone() {
                    DragState::PanCanvas { start_mouse, start_pan } => {
                        let dx = mx - start_mouse.0;
                        let dy = my - start_mouse.1;
                        st.pan = (start_pan.0 + dx, start_pan.1 + dy);
                        cx.notify();
                    }
                    DragState::DragNodes { start_mouse, initial_positions } => {
                        let dx = (mx - start_mouse.0) / st.zoom;
                        let dy = (my - start_mouse.1) / st.zoom;

                        let selected_id = st.selected_node_id.clone();
                        for (id, orig_pos) in initial_positions {
                            if selected_id.as_deref() == Some(id) {
                                if let Some(n) = st.graph.find_node_mut(id) {
                                    n.pos = (orig_pos.0 + dx, orig_pos.1 + dy);
                                }
                            }
                        }
                        cx.notify();
                    }
                    DragState::Connecting { source_node_id, source_port_id, direction, source_pos, .. } => {
                        st.drag_state = DragState::Connecting {
                            source_node_id: source_node_id.clone(),
                            source_port_id: source_port_id.clone(),
                            direction: *direction,
                            source_pos: *source_pos,
                            current_mouse: (mx, my),
                        };
                        cx.notify();
                    }
                    DragState::None => {}
                }
            });
        })
        // Mouse up on empty canvas
        .on_mouse_up(MouseButton::Left, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_mouse_up.update(cx, |st, cx| {
                if let DragState::Connecting { source_node_id, source_port_id, direction, .. } = &st.drag_state.clone() {
                    // 松开连线在空白画布上时，弹出快速新建节点浮层
                    let (canvas_x, canvas_y) = st.screen_to_canvas(mx, my);
                    st.quick_add_modal = Some((canvas_x, canvas_y, Some((source_node_id.clone(), source_port_id.clone(), *direction))));
                }
                st.drag_state = DragState::None;
                cx.notify();
            });
        })

        // Scroll wheel to zoom
        .on_scroll_wheel(move |e, _, cx| {
            let dy: f32 = match e.delta {
                ScrollDelta::Pixels(pt) => pt.y.into(),
                ScrollDelta::Lines(pt) => pt.y * 20.0,
            };
            let zoom_factor = if dy > 0.0 { 1.08 } else { 0.92 };

            model_scroll.update(cx, |st, cx| {
                let old_zoom = st.zoom;
                let new_zoom = (old_zoom * zoom_factor).clamp(0.25, 2.5);

                // Zoom towards center of canvas
                let cx_center = 550.0;
                let cy_center = 350.0;

                st.pan.0 = cx_center - (cx_center - st.pan.0) * (new_zoom / old_zoom);
                st.pan.1 = cy_center - (cy_center - st.pan.1) * (new_zoom / old_zoom);
                st.zoom = new_zoom;

                cx.notify();
            });
        })
        // 1. Background Grid
        .child(
            canvas(
                |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    draw_canvas_grid(bounds, pan, zoom, window);
                },
            )
            .size_full(),
        )
        // 2. Semantic Groups Backdrop Layer
        .child(render_groups_layer(state))
        // 3. Edges Layer
        .child(render_edges_layer(state, model))
        // 4. Nodes Layer
        .child(render_nodes_layer(state, model))
        // 5. Quick Add modal overlay if active
        .children(quick_add.map(|(cx, cy, src)| {
            render_quick_add_modal(state, (cx, cy), src, model)
        }))
}


