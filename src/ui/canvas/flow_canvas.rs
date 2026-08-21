use gpui::*;
use crate::ui::canvas::edges::render_edges_layer;
use crate::ui::canvas::grid::draw_canvas_grid;
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
        .bg(hsla(220.0 / 360.0, 0.35, 0.08, 1.0))
        .cursor(match &drag_state {
            DragState::PanCanvas { .. } => CursorStyle::ClosedHand,
            DragState::Connecting { .. } => CursorStyle::Crosshair,
            _ => CursorStyle::Arrow,
        })
        // Mouse down on empty canvas
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_mouse_down.update(cx, |st, cx| {
                st.selected_node_id = None;
                st.quick_add_modal = None;
                st.drag_state = DragState::PanCanvas {
                    start_mouse: (mx, my),
                    start_pan: st.pan,
                };
                cx.notify();
            });
        })
        // Right click to open Quick Add modal at cursor
        .on_mouse_down(MouseButton::Right, {
            let model_right = model.clone();
            move |e, _, cx| {
                let mx: f32 = e.position.x.into();
                let my: f32 = e.position.y.into();
                model_right.update(cx, |st, cx| {
                    let (canvas_x, canvas_y) = st.screen_to_canvas(mx, my);
                    st.quick_add_modal = Some((canvas_x, canvas_y, None));
                    cx.notify();
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
                    DragState::Connecting { source_node_id, source_port_id, source_pos, .. } => {
                        st.drag_state = DragState::Connecting {
                            source_node_id: source_node_id.clone(),
                            source_port_id: source_port_id.clone(),
                            source_pos: *source_pos,
                            current_mouse: (mx, my),
                        };
                        cx.notify();
                    }
                    DragState::None => {}
                }
            });
        })
        // Mouse up
        .on_mouse_up(MouseButton::Left, move |e, _, cx| {
            let mx: f32 = e.position.x.into();
            let my: f32 = e.position.y.into();
            model_mouse_up.update(cx, |st, cx| {
                if let DragState::Connecting { source_node_id, source_port_id, .. } = &st.drag_state.clone() {
                    // Releasing connecting wire on empty canvas pops up Quick Add
                    let (canvas_x, canvas_y) = st.screen_to_canvas(mx, my);
                    st.quick_add_modal = Some((canvas_x, canvas_y, Some((source_node_id.clone(), source_port_id.clone()))));
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
        // Background Grid
        .child(
            canvas(
                |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    draw_canvas_grid(bounds, pan, zoom, window);
                },
            )
            .size_full(),
        )
        // Edges layer
        .child(render_edges_layer(state, model))
        // Nodes layer
        .child(render_nodes_layer(state, model))
        // Quick Add modal overlay if active
        .children(quick_add.map(|(cx, cy, src)| {
            render_quick_add_modal((cx, cy), src, model)
        }))
}
