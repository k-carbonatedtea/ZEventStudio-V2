use crate::{model::map_editor::MapDocumentState, ui::state::AppState};
use gpui::*;
use std::sync::{Arc, Mutex};
use uking_library::map::Transform;

pub fn render_viewport(map: &MapDocumentState, model: &Entity<AppState>, id: &str) -> AnyElement {
    let rect = Arc::new(Mutex::new([0f32, 0., 1., 1.]));
    let paint_rect = rect.clone();
    let displayed_camera = Arc::new(Mutex::new(None));
    let painted_camera = displayed_camera.clone();
    let snapshot = map.clone();
    let mut view = div()
        .id("map-viewport")
        .flex_1()
        .min_w_0()
        .relative()
        .overflow_hidden()
        .bg(rgb(0x172027))
        .child(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, _| {
                    *paint_rect.lock().unwrap() = [
                        bounds.origin.x.into(),
                        bounds.origin.y.into(),
                        bounds.size.width.into(),
                        bounds.size.height.into(),
                    ];
                    if !snapshot.loading {
                        let scale = window.scale_factor();
                        let (frame, pending) = super::async_render::request(
                            &snapshot,
                            f32::from(bounds.size.width) * scale,
                            f32::from(bounds.size.height) * scale,
                        );
                        if let Some(frame) = frame {
                            if let Some(image) = &frame.image {
                                crate::ui::views::model_preview::renderer::paint_rendered_image(
                                    image.clone(),
                                    bounds,
                                    window,
                                );
                                *painted_camera.lock().unwrap() =
                                    Some(frame.map.scene.camera.clone());
                                draw_area_outlines(&frame.map, bounds, window);
                                draw_selection(&frame.map, bounds, window);
                                draw_links(
                                    &frame.map,
                                    snapshot.show_links,
                                    snapshot.selected_links_only,
                                    bounds,
                                    window,
                                );
                            }
                        }
                        if pending {
                            window.request_animation_frame();
                        }
                    }
                },
            )
            .size_full(),
        );
    for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
        let model = model.clone();
        let id = id.to_string();
        view = view.on_mouse_down(button, move |e, _, cx| {
            model.update(cx, |st, cx| {
                st.clear_active_text_inputs();
                if st
                    .active_map()
                    .is_some_and(|m| m.input.is_some() && m.input_error.is_some())
                {
                    return;
                }
                st.update_map(&id, |m| {
                    let pos = (f32::from(e.position.x), f32::from(e.position.y));
                    m.last_mouse = pos;
                    m.press_mouse = pos;
                    m.orbiting = button == MouseButton::Right
                        || (button == MouseButton::Left && e.modifiers.alt);
                    m.panning = button == MouseButton::Middle;
                    m.dragging =
                        button == MouseButton::Left && e.modifiers.shift && m.selected.is_some();
                    m.drag_changed = false;
                    if m.dragging {
                        m.checkpoint();
                    }
                    Ok(())
                });
                cx.notify();
            });
        });
    }
    let m = model.clone();
    let tid = id.to_string();
    let r = rect.clone();
    view = view.on_mouse_move(move |e, _, cx| {
        m.update(cx, |st, cx| {
            st.update_map(&tid, |map| {
                let pos = (f32::from(e.position.x), f32::from(e.position.y));
                let dx = pos.0 - map.last_mouse.0;
                let dy = pos.1 - map.last_mouse.1;
                let rect = *r.lock().unwrap();
                if map.orbiting {
                    map.scene.camera.orbit(dx, dy);
                } else if map.panning {
                    map.scene.camera.pan(dx, dy, rect[3]);
                } else if map.dragging {
                    if let Some(index) = map.selected {
                        let mut transform = Transform::from_actor(&map.file.actors()[index]);
                        if let (Some(a), Some(b)) = (
                            plane_point(
                                &map.scene.camera,
                                map.last_mouse,
                                rect,
                                transform.translation[1],
                            ),
                            plane_point(&map.scene.camera, pos, rect, transform.translation[1]),
                        ) {
                            transform.translation[0] += b[0] - a[0];
                            transform.translation[2] += b[2] - a[2];
                            map.file.set_transform(index, transform)?;
                            map.rebuild_instances();
                            map.drag_changed = true;
                        }
                    }
                }
                map.last_mouse = pos;
                Ok(())
            });
            cx.notify();
        });
    });
    for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
        let mouse_up_model = model.clone();
        let tid = id.to_string();
        let r = rect.clone();
        let displayed_camera = displayed_camera.clone();
        view = view.on_mouse_up(button, move |e, _, cx| {
            mouse_up_model.update(cx, |st, cx| {
                st.update_map(&tid, |m| {
                    if m.input.is_some() && m.input_error.is_some() {
                        return Ok(());
                    }
                    let pos = (f32::from(e.position.x), f32::from(e.position.y));
                    if button == MouseButton::Left
                        && !m.orbiting
                        && !m.drag_changed
                        && (pos.0 - m.press_mouse.0).hypot(pos.1 - m.press_mouse.1) < 4.
                    {
                        let r = *r.lock().unwrap();
                        let camera = displayed_camera.lock().unwrap();
                        let hit = m.pick_with_camera(
                            camera.as_ref().unwrap_or(&m.scene.camera),
                            pos.0 - r[0],
                            pos.1 - r[1],
                            r[2],
                            r[3],
                        );
                        m.select_ref(hit);
                    }
                    m.orbiting = false;
                    m.panning = false;
                    m.dragging = false;
                    Ok(())
                });
                cx.notify();
            });
        });
        let model = model.clone();
        let tid = id.to_string();
        view = view.on_mouse_up_out(button, move |_, _, cx| {
            model.update(cx, |st, cx| {
                st.update_map(&tid, |m| {
                    m.orbiting = false;
                    m.panning = false;
                    m.dragging = false;
                    Ok(())
                });
                cx.notify();
            });
        });
    }
    let m = model.clone();
    let tid = id.to_string();
    view = view.on_scroll(move |e, _, cx| {
        let delta = match e.delta {
            ScrollDelta::Lines(l) => l.y * 18.,
            ScrollDelta::Pixels(p) => p.y.into(),
        };
        m.update(cx, |st, cx| {
            st.update_map(&tid, |m| {
                m.scene.camera.zoom((-delta * 0.003).exp().clamp(0.5, 2.));
                Ok(())
            });
            cx.notify();
        });
    });
    if map.loading {
        view = view.child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(0xB5C8D3))
                .child("正在解析对象模型、纹理与地形…"),
        );
    }
    view.into_any_element()
}

fn plane_point(
    camera: &crate::model::model_preview::OrbitCamera,
    pos: (f32, f32),
    rect: [f32; 4],
    height: f32,
) -> Option<[f32; 3]> {
    let (right, up, forward) = camera.compute_basis();
    let tan = (camera.fov / 2.).tan();
    let x = (2. * (pos.0 - rect[0]) / rect[2] - 1.) * rect[2] / rect[3] * tan;
    let y = (1. - 2. * (pos.1 - rect[1]) / rect[3]) * tan;
    let direction: [f32; 3] = std::array::from_fn(|i| forward[i] + right[i] * x + up[i] * y);
    let eye = camera.compute_eye();
    if direction[1].abs() < 0.00001 {
        return None;
    }
    let t = (height - eye[1]) / direction[1];
    if t < 0. {
        return None;
    }
    Some(std::array::from_fn(|i| eye[i] + direction[i] * t))
}

fn draw_selection(map: &MapDocumentState, bounds: Bounds<Pixels>, window: &mut Window) {
    let Some(reference) = map.selected_ref() else {
        return;
    };
    let Some(actor) = map.actor(reference) else {
        return;
    };
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let transform = Transform::from_actor(actor);
    let projector = map.scene.camera.projector(w, h);
    let mut path = PathBuilder::stroke(px(1.5));
    let mut count = 0;
    if let Some(instances) = &map.scene.instances {
        for (instance, owner) in instances.iter().zip(map.instance_owners.iter()) {
            if *owner != Some(reference) {
                continue;
            }
            let b = &map.scene.meshes[instance.mesh_index].bounding_box;
            let lo = b.min();
            let hi = b.max();
            let corners = [
                [lo.x, lo.y, lo.z],
                [hi.x, lo.y, lo.z],
                [hi.x, hi.y, lo.z],
                [lo.x, hi.y, lo.z],
                [lo.x, lo.y, hi.z],
                [hi.x, lo.y, hi.z],
                [hi.x, hi.y, hi.z],
                [lo.x, hi.y, hi.z],
            ]
            .map(|p| transform.point(p));
            for (a, b) in [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ] {
                if let (Some((a, _)), Some((b, _))) =
                    (projector.project(corners[a]), projector.project(corners[b]))
                {
                    path.move_to(point(px(ox + a[0]), px(oy + a[1])));
                    path.line_to(point(px(ox + b[0]), px(oy + b[1])));
                    count += 1;
                }
            }
        }
    }
    if count > 0 {
        if let Ok(path) = path.build() {
            window.paint_path(path, rgb(0x63DCCB));
        }
    }
    crate::ui::views::model_preview::gizmos::draw_axes(
        &map.scene.camera,
        transform.translation,
        (map.scene.camera.distance * 0.04).clamp(1., 20.),
        ox,
        oy,
        w,
        h,
        window,
    );
}

fn draw_area_outlines(map: &MapDocumentState, bounds: Bounds<Pixels>, window: &mut Window) {
    let projector = map
        .scene
        .camera
        .projector(bounds.size.width.into(), bounds.size.height.into());
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let mut path = PathBuilder::stroke(px(1.2));
    let mut selected_path = PathBuilder::stroke(px(2.0));
    let mut count = 0;
    let mut selected_count = 0;
    let mut draw_one = |reference: crate::model::map_editor::ActorRef, actor: &roead::byml::Byml| {
        if !map.assets.area_actors.contains(uking_library::map::actor_name(actor))
            || !map.object_visible(reference)
        {
            return;
        }
        let t = Transform::from_actor(actor);
        let corners = [
            [-1., -1., -1.],
            [1., -1., -1.],
            [1., 1., -1.],
            [-1., 1., -1.],
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., 1.],
            [-1., 1., 1.],
        ]
        .map(|p| t.point(p));
        let selected = map.selected_ref() == Some(reference);
        let target = if selected { &mut selected_path } else { &mut path };
        for (a, b) in [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ] {
            if let (Some((a, _)), Some((b, _))) = (projector.project(corners[a]), projector.project(corners[b])) {
                target.move_to(point(px(ox + a[0]), px(oy + a[1])));
                target.line_to(point(px(ox + b[0]), px(oy + b[1])));
                if selected { selected_count += 1; } else { count += 1; }
            }
        }
    };
    for (i, actor) in map.file.actors().iter().enumerate() {
        draw_one(crate::model::map_editor::ActorRef::Current(i), actor);
    }
    for (i, actor) in map.assets.context_actors.iter().enumerate() {
        draw_one(crate::model::map_editor::ActorRef::Context(i), actor);
    }
    if count > 0 {
        if let Ok(path) = path.build() { window.paint_path(path, rgb(0x111820)); }
    }
    if selected_count > 0 {
        if let Ok(path) = selected_path.build() { window.paint_path(path, rgb(0x63DCCB)); }
    }
}

fn draw_links(
    map: &MapDocumentState,
    show: bool,
    selected_only: bool,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    if !show {
        return;
    }
    let projector = map
        .scene
        .camera
        .projector(bounds.size.width.into(), bounds.size.height.into());
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    for selected in [false, true] {
        let mut path = PathBuilder::stroke(px(if selected { 2. } else { 1. }));
        let mut count = 0;
        for link in map.links.iter() {
            let connected = map
                .selected_ref()
                .is_some_and(|r| link.source == r || link.target == Some(r));
            if connected != selected || (selected_only && !connected) {
                continue;
            }
            if !map.object_visible(link.source)
                || link.target.is_some_and(|r| !map.object_visible(r))
            {
                continue;
            }
            if !map.show_context
                && (matches!(link.source, crate::model::map_editor::ActorRef::Context(_))
                    || matches!(
                        link.target,
                        Some(crate::model::map_editor::ActorRef::Context(_))
                    ))
            {
                continue;
            }
            let Some(to) = link.to else {
                continue;
            };
            let (Some((a, _)), Some((b, _))) =
                (projector.project(link.from), projector.project(to))
            else {
                continue;
            };
            let [ax, ay] = [ox + a[0], oy + a[1]];
            let [bx, by] = [ox + b[0], oy + b[1]];
            path.move_to(point(px(ax), px(ay)));
            path.line_to(point(px(bx), px(by)));
            let dx = bx - ax;
            let dy = by - ay;
            let len = dx.hypot(dy);
            if len > 12. {
                let size = 7.;
                let ux = dx / len;
                let uy = dy / len;
                path.move_to(point(
                    px(bx - size * ux + size * 0.45 * uy),
                    px(by - size * uy - size * 0.45 * ux),
                ));
                path.line_to(point(px(bx), px(by)));
                path.line_to(point(
                    px(bx - size * ux - size * 0.45 * uy),
                    px(by - size * uy + size * 0.45 * ux),
                ));
            }
            count += 1;
        }
        if count > 0 {
            if let Ok(path) = path.build() {
                window.paint_path(
                    path,
                    if selected {
                        rgba(0xF2C46EFF)
                    } else {
                        rgba(0x7B9CA699)
                    },
                );
            }
        }
    }
}
