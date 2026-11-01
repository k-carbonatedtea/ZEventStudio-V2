//! UMii 编辑器内嵌 3D 模型合成视口画布与相机手势交互

use gpui::*;
use crate::model::model_preview::ModelDocumentState;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;
use crate::ui::views::model_preview::renderer::draw_model_viewport;

pub fn render_umii_viewport(
    model_doc: Option<&ModelDocumentState>,
    tab_id: &str,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    state: &AppState,
) -> impl IntoElement {
    let Some(doc) = model_doc else {
        return div()
            .flex_1()
            .size_full()
            .bg(pal.card_surface)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .child(div().text_size(px(32.0)).child("👤"))
            .child(
                div()
                    .text_xs()
                    .text_color(pal.text_secondary)
                    .child(state.t("umii.preview_3d")),
            )
            .into_any_element();
    };

    let tid = tab_id.to_string();
    let m_down_left = model.clone();
    let tid_down_left = tid.clone();

    let m_down_right = model.clone();
    let tid_down_right = tid.clone();

    let m_move = model.clone();
    let tid_move = tid.clone();

    let m_up_left = model.clone();
    let tid_up_left = tid.clone();

    let m_up_right = model.clone();
    let tid_up_right = tid.clone();

    let m_scroll = model.clone();
    let tid_scroll = tid.clone();

    let doc_paint = doc.clone();
    let total_v = doc.total_vertices;
    let total_t = doc.total_triangles;
    let mesh_count = doc.mesh_items.len();
    let is_switch = doc.is_switch;

    div()
        .flex_1()
        .size_full()
        .relative()
        .overflow_hidden()
        .bg(rgb(0x0B0F19))
        // 1. 3D 核心渲染 Canvas
        .child(
            canvas(
                |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    draw_model_viewport(&doc_paint, bounds, window);
                },
            )
            .size_full(),
        )
        // 2. 鼠标手势交互捕获层
        .child(
            div()
                .absolute()
                .inset_0()
                .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    let pos = (event.position.x.into(), event.position.y.into());
                    let tid = tid_down_left.clone();
                    m_down_left.update(cx, |st, cx| {
                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                            if let Some(ref mut u) = t.umii_state {
                                if let Some(ref mut m) = u.synthesized_model {
                                    m.is_orbiting = true;
                                    m.last_mouse_pos = pos;
                                }
                            }
                        }
                        cx.notify();
                    });
                })
                .on_mouse_down(MouseButton::Right, move |event, _, cx| {
                    let pos = (event.position.x.into(), event.position.y.into());
                    let tid = tid_down_right.clone();
                    m_down_right.update(cx, |st, cx| {
                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                            if let Some(ref mut u) = t.umii_state {
                                if let Some(ref mut m) = u.synthesized_model {
                                    m.is_panning = true;
                                    m.last_mouse_pos = pos;
                                }
                            }
                        }
                        cx.notify();
                    });
                })
                .on_mouse_move(move |event, _, cx| {
                    let current_pos = (event.position.x.into(), event.position.y.into());
                    let tid = tid_move.clone();
                    m_move.update(cx, |st, cx| {
                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                            if let Some(ref mut u) = t.umii_state {
                                if let Some(ref mut m) = u.synthesized_model {
                                    if m.is_orbiting {
                                        let dx = current_pos.0 - m.last_mouse_pos.0;
                                        let dy = current_pos.1 - m.last_mouse_pos.1;
                                        m.camera.orbit(dx, dy);
                                        m.last_mouse_pos = current_pos;
                                        cx.notify();
                                    } else if m.is_panning {
                                        let dx = current_pos.0 - m.last_mouse_pos.0;
                                        let dy = current_pos.1 - m.last_mouse_pos.1;
                                        m.camera.pan(dx, dy, 600.0);
                                        m.last_mouse_pos = current_pos;
                                        cx.notify();
                                    }
                                }
                            }
                        }
                    });
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    let tid = tid_up_left.clone();
                    m_up_left.update(cx, |st, cx| {
                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                            if let Some(ref mut u) = t.umii_state {
                                if let Some(ref mut m) = u.synthesized_model {
                                    m.is_orbiting = false;
                                }
                            }
                        }
                        cx.notify();
                    });
                })
                .on_mouse_up(MouseButton::Right, move |_, _, cx| {
                    let tid = tid_up_right.clone();
                    m_up_right.update(cx, |st, cx| {
                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                            if let Some(ref mut u) = t.umii_state {
                                if let Some(ref mut m) = u.synthesized_model {
                                    m.is_panning = false;
                                }
                            }
                        }
                        cx.notify();
                    });
                })
                .on_scroll(move |event, _, cx| {
                    let delta_y = match event.delta {
                        ScrollDelta::Lines(lines) => lines.y * 30.0,
                        ScrollDelta::Pixels(pixels) => pixels.y.into(),
                    };
                    let tid = tid_scroll.clone();
                    m_scroll.update(cx, |st, cx| {
                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                            if let Some(ref mut u) = t.umii_state {
                                if let Some(ref mut m) = u.synthesized_model {
                                    m.camera.zoom(delta_y);
                                    cx.notify();
                                }
                            }
                        }
                    });
                }),
        )
        // 3. 悬浮 HUD 状态信息
        .child(
            div()
                .absolute()
                .top_3()
                .left_3()
                .px_2p5()
                .py_1p5()
                .rounded_md()
                .bg(rgb(0x111827).opacity(0.85))
                .border_1()
                .border_color(rgb(0x374151))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(if is_switch { rgb(0x1E3A8A) } else { rgb(0x065F46) })
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .child(if is_switch { "Switch NX" } else { "Wii U Cafe" }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xF9FAFB))
                                .child(format!("{} 部件合成", mesh_count)),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9CA3AF))
                        .child(format!("顶点: {} | 三角面: {}", total_v, total_t)),
                ),
        )
        .into_any_element()
}
