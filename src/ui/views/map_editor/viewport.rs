use crate::{
    model::{
        map_editor::{LinkTargetKind, MapDocumentState, MapViewMode},
        model_preview::OrbitCamera,
    },
    ui::state::AppState,
};
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
                move |bounds, _, window, cx| {
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
                                *painted_camera.lock().unwrap() = Some(frame.camera.clone());
                                if snapshot.show_hidden_perspective {
                                    draw_area_outlines(&snapshot, &frame.camera, bounds, window);
                                }
                                draw_selection(&snapshot, &frame.camera, bounds, window);
                                draw_links(
                                    &snapshot,
                                    &frame.camera,
                                    snapshot.show_links,
                                    snapshot.selected_links_only,
                                    bounds,
                                    window,
                                    cx,
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
        let mut visual_changed = false;
        m.update(cx, |st, cx| {
            st.update_map(&tid, |map| {
                let pos = (f32::from(e.position.x), f32::from(e.position.y));
                let dx = pos.0 - map.last_mouse.0;
                let dy = pos.1 - map.last_mouse.1;
                let rect = *r.lock().unwrap();
                let moved = dx != 0.0 || dy != 0.0;
                if map.orbiting && moved {
                    map.scene.camera.orbit(dx, dy);
                    visual_changed = true;
                } else if map.panning && moved {
                    map.scene.camera.pan(dx, dy, rect[3]);
                    visual_changed = true;
                } else if map.dragging && moved {
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
                            visual_changed = true;
                        }
                    }
                }
                map.last_mouse = pos;
                Ok(())
            });
            // Updating the hover position is needed for the next drag, but does
            // not change anything painted. Avoid rebuilding the whole editor
            // while the pointer merely passes over the viewport.
            if visual_changed {
                cx.notify();
            }
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

fn draw_selection(
    map: &MapDocumentState,
    camera: &OrbitCamera,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    let Some(reference) = map.selected_ref() else {
        return;
    };
    let Some(actor) = map.actor(reference) else {
        return;
    };
    if !map.object_visible(reference) {
        return;
    }
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let transform = Transform::from_actor(actor);
    let projector = camera.projector(w, h);
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
        camera,
        transform.translation,
        (camera.distance * 0.04).clamp(1., 20.),
        ox,
        oy,
        w,
        h,
        window,
    );
}

fn draw_area_outlines(
    map: &MapDocumentState,
    camera: &OrbitCamera,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    let projector = camera.projector(bounds.size.width.into(), bounds.size.height.into());
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let mut path = PathBuilder::stroke(px(0.85));
    let mut selected_path = PathBuilder::stroke(px(2.0));
    let mut count = 0;
    let mut selected_count = 0;
    let mut detail_count = 0usize;
    const MIN_OUTLINE_SIZE: f32 = 28.0;
    const MAX_OUTLINES: usize = 192;
    let selected_reference = map.selected_ref();
    let mut draw_one = |reference: crate::model::map_editor::ActorRef,
                        actor: &roead::byml::Byml| {
        if !map
            .assets
            .area_actors
            .contains(uking_library::map::actor_name(actor))
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
        let selected = selected_reference == Some(reference);
        let projected = corners.map(|corner| projector.project(corner));
        if !selected {
            let Some((first, _)) = projected.iter().flatten().next() else {
                return;
            };
            let (mut min_x, mut max_x) = (first[0], first[0]);
            let (mut min_y, mut max_y) = (first[1], first[1]);
            for (screen, _) in projected.iter().flatten().skip(1) {
                min_x = min_x.min(screen[0]);
                max_x = max_x.max(screen[0]);
                min_y = min_y.min(screen[1]);
                max_y = max_y.max(screen[1]);
            }
            if (max_x - min_x).max(max_y - min_y) < MIN_OUTLINE_SIZE || detail_count >= MAX_OUTLINES
            {
                return;
            }
            detail_count += 1;
        }
        let target = if selected {
            &mut selected_path
        } else {
            &mut path
        };
        let mut add_segment = |a: [f32; 3], b: [f32; 3]| {
            if let (Some((a, _)), Some((b, _))) = (projector.project(a), projector.project(b)) {
                target.move_to(point(px(ox + a[0]), px(oy + a[1])));
                target.line_to(point(px(ox + b[0]), px(oy + b[1])));
                if selected {
                    selected_count += 1;
                } else {
                    count += 1;
                }
            }
        };
        let shape = actor
            .as_map()
            .ok()
            .and_then(|m| m.get("!Parameters"))
            .and_then(|p| p.as_map().ok())
            .and_then(|p| p.get("Shape"))
            .and_then(|v| v.as_string().ok())
            .map(|v| v.as_str())
            .unwrap_or("Box");
        match shape {
            "Sphere" => {
                const SEGMENTS: usize = 12;
                for axis in 0..3 {
                    let mut ring = [[0.; 3]; SEGMENTS + 1];
                    for (index, point) in ring.iter_mut().enumerate() {
                        let angle = index as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
                        let (sin, cos) = angle.sin_cos();
                        *point = match axis {
                            0 => [0., cos, sin],
                            1 => [cos, 0., sin],
                            _ => [cos, sin, 0.],
                        };
                        *point = t.point(*point);
                    }
                    for pair in ring.windows(2) {
                        add_segment(pair[0], pair[1]);
                    }
                }
            }
            "Cylinder" => {
                const SEGMENTS: usize = 12;
                let mut top = [[0.; 3]; SEGMENTS + 1];
                let mut bottom = [[0.; 3]; SEGMENTS + 1];
                for index in 0..=SEGMENTS {
                    let angle = index as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
                    let (sin, cos) = angle.sin_cos();
                    top[index] = t.point([cos, 1., sin]);
                    bottom[index] = t.point([cos, -1., sin]);
                }
                for pair in top.windows(2) {
                    add_segment(pair[0], pair[1]);
                }
                for pair in bottom.windows(2) {
                    add_segment(pair[0], pair[1]);
                }
                for index in [0, SEGMENTS / 4, SEGMENTS / 2, SEGMENTS * 3 / 4] {
                    add_segment(top[index], bottom[index]);
                }
            }
            _ => {
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
                    if let (Some((a, _)), Some((b, _))) = (projected[a], projected[b]) {
                        target.move_to(point(px(ox + a[0]), px(oy + a[1])));
                        target.line_to(point(px(ox + b[0]), px(oy + b[1])));
                        if selected {
                            selected_count += 1;
                        } else {
                            count += 1;
                        }
                    }
                }
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
        if let Ok(path) = path.build() {
            window.paint_path(path, rgba(0x11182088));
        }
    }
    if selected_count > 0 {
        if let Ok(path) = selected_path.build() {
            window.paint_path(path, rgb(0x63DCCB));
        }
    }
}

const LINK_KINDS: [LinkTargetKind; 8] = LinkTargetKind::ALL;

fn system_node_position(
    kind: LinkTargetKind,
    ox: f32,
    oy: f32,
    width: f32,
    height: f32,
) -> [f32; 2] {
    let step = ((height - 36.) / 8.).clamp(10., 25.);
    [ox + width - 16., oy + 20. + kind.index() as f32 * step]
}

fn link_color(kind: LinkTargetKind, selected: bool) -> Rgba {
    rgba((kind.color() << 8) | if selected { 0xEE } else { 0x44 })
}

fn draw_system_node(path: &mut PathBuilder, kind: LinkTargetKind, x: f32, y: f32) {
    let size = 5.;
    match kind {
        LinkTargetKind::Camera => {
            path.move_to(point(px(x), px(y - size)));
            path.line_to(point(px(x + size), px(y + size)));
            path.line_to(point(px(x - size), px(y + size)));
            path.close();
        }
        LinkTargetKind::Event => {
            path.move_to(point(px(x), px(y - size)));
            path.line_to(point(px(x + size), px(y)));
            path.line_to(point(px(x), px(y + size)));
            path.line_to(point(px(x - size), px(y)));
            path.close();
        }
        LinkTargetKind::Weather => {
            for index in 0..=8 {
                let angle = index as f32 / 8. * std::f32::consts::TAU;
                let (sin, cos) = angle.sin_cos();
                let p = point(px(x + cos * size), px(y + sin * size));
                if index == 0 {
                    path.move_to(p);
                } else {
                    path.line_to(p);
                }
            }
            path.close();
        }
        LinkTargetKind::Audio
        | LinkTargetKind::External
        | LinkTargetKind::Unknown
        | LinkTargetKind::MapObject
        | LinkTargetKind::Physics => {
            path.move_to(point(px(x - size), px(y - size)));
            path.line_to(point(px(x + size), px(y - size)));
            path.line_to(point(px(x + size), px(y + size)));
            path.line_to(point(px(x - size), px(y + size)));
            path.close();
        }
    }
}

fn add_link_segment(path: &mut PathBuilder, kind: LinkTargetKind, a: [f32; 2], b: [f32; 2]) {
    let pattern: Option<(f32, f32)> = match kind {
        LinkTargetKind::Camera => Some((10., 6.)),
        LinkTargetKind::Weather => Some((3., 5.)),
        LinkTargetKind::External | LinkTargetKind::Unknown => Some((5., 5.)),
        _ => None,
    };
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let length = dx.hypot(dy);
    if let Some((dash, gap)) = pattern.filter(|_| length > 1.) {
        let period = (dash + gap).max(length / 64.);
        let mut offset = 0.;
        while offset < length {
            let start = offset / length;
            let end = ((offset + period * dash / (dash + gap)) / length).min(1.);
            path.move_to(point(px(a[0] + dx * start), px(a[1] + dy * start)));
            path.line_to(point(px(a[0] + dx * end), px(a[1] + dy * end)));
            offset += period;
        }
    } else {
        path.move_to(point(px(a[0]), px(a[1])));
        path.line_to(point(px(b[0]), px(b[1])));
    }
}

fn draw_links(
    map: &MapDocumentState,
    camera: &OrbitCamera,
    show: bool,
    selected_only: bool,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    if !show || map.view_mode == MapViewMode::Game {
        return;
    }
    let selected = map.selected_ref();
    if selected_only && selected.is_none() {
        return;
    }
    let projector = camera.projector(bounds.size.width.into(), bounds.size.height.into());
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let mut normal_paths = LINK_KINDS.map(|_| PathBuilder::stroke(px(1.)));
    let mut selected_paths = LINK_KINDS.map(|_| PathBuilder::stroke(px(2.)));
    let mut system_nodes = LINK_KINDS.map(|_| PathBuilder::fill());
    let mut selected_nodes = PathBuilder::stroke(px(2.));
    let mut source_points = PathBuilder::fill();
    let mut target_points = PathBuilder::fill();
    let mut normal_counts = [0usize; 8];
    let mut selected_counts = [0usize; 8];
    let mut node_drawn = [false; 8];
    let mut unresolved_counts = [0usize; 8];
    let mut reference_counts = [0usize; 8];
    let link_count = if selected_only {
        map.selected_link_indices.len()
    } else {
        map.links.len()
    };
    for index in 0..link_count {
        let index = if selected_only {
            map.selected_link_indices[index]
        } else {
            index
        };
        let link = &map.links[index];
        let connected = selected.is_some_and(|r| link.source == r || link.target == Some(r));
        if (selected_only && !connected)
            || !map.category_matches(link.target_kind)
            || (link.target.is_none() && !connected)
        {
            continue;
        }
        if !map.object_visible(link.source) || link.target.is_some_and(|r| !map.object_visible(r)) {
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
        let Some((a, _)) = projector.project(link.from) else {
            continue;
        };
        let [ax, ay] = [ox + a[0], oy + a[1]];
        // Category says what a link means, not whether its target has a
        // location. Resolved camera/event/weather actors keep world endpoints.
        let (bx, by) = if let Some(to) = link.to {
            let Some((b, _)) = projector.project(to) else {
                continue;
            };
            (ox + b[0], oy + b[1])
        } else {
            let [x, y] = system_node_position(link.target_kind, ox, oy, width, height);
            (x, y)
        };
        if ((ax < ox && bx < ox)
            || (ax > ox + width && bx > ox + width)
            || (ay < oy && by < oy)
            || (ay > oy + height && by > oy + height))
            || ![ax, ay, bx, by].iter().all(|v| v.is_finite())
        {
            continue;
        }
        let kind_index = link.target_kind.index();
        let path = if connected {
            selected_counts[kind_index] += 1;
            &mut selected_paths[kind_index]
        } else {
            normal_counts[kind_index] += 1;
            &mut normal_paths[kind_index]
        };
        add_link_segment(path, link.target_kind, [ax, ay], [bx, by]);
        if link.target.is_none() {
            unresolved_counts[kind_index] += 1;
            if !node_drawn[kind_index] {
                draw_system_node(&mut system_nodes[kind_index], link.target_kind, bx, by);
                node_drawn[kind_index] = true;
            }
        }
        if connected {
            draw_system_node(&mut selected_nodes, link.target_kind, bx, by);
            let size = 6.;
            for (builder, x, y) in [(&mut source_points, ax, ay), (&mut target_points, bx, by)] {
                let radius = if x == bx && y == by { 1.5 } else { size * 0.5 };
                builder.move_to(point(px(x - radius), px(y - radius)));
                builder.line_to(point(px(x + radius), px(y - radius)));
                builder.line_to(point(px(x + radius), px(y + radius)));
                builder.line_to(point(px(x - radius), px(y + radius)));
                builder.close();
            }
        }
    }
    // Parameter references are a different relationship from LinksToObj.
    // Aggregate their lines by category and list concrete names in the inspector.
    if let Some((reference, actor)) = selected.and_then(|r| map.actor(r).map(|a| (r, a))) {
        if map.object_visible(reference) {
            if let Some((a, _)) = projector.project(Transform::from_actor(actor).translation) {
                for binding in map
                    .object_system_references(reference)
                    .filter(|r| map.category_matches(r.kind))
                {
                    reference_counts[binding.kind.index()] += 1;
                }
                for kind in LINK_KINDS {
                    let i = kind.index();
                    if reference_counts[i] == 0 {
                        continue;
                    }
                    let [x, y] = system_node_position(kind, ox, oy, width, height);
                    add_link_segment(
                        &mut selected_paths[i],
                        LinkTargetKind::External,
                        [ox + a[0], oy + a[1]],
                        [x, y],
                    );
                    selected_counts[i] += 1;
                    if !node_drawn[i] {
                        draw_system_node(&mut system_nodes[i], kind, x, y);
                        node_drawn[i] = true;
                    }
                }
            }
        }
    }
    for (index, (((kind, normal_path), selected_path), system_node)) in LINK_KINDS
        .into_iter()
        .zip(normal_paths)
        .zip(selected_paths)
        .zip(system_nodes)
        .enumerate()
    {
        if normal_counts[index] > 0 {
            if let Ok(path) = normal_path.build() {
                window.paint_path(path, link_color(kind, false));
            }
        }
        if selected_counts[index] > 0 {
            if let Ok(path) = selected_path.build() {
                window.paint_path(path, link_color(kind, true));
            }
        }
        if node_drawn[index] {
            if let Ok(path) = system_node.build() {
                window.paint_path(path, link_color(kind, false));
            }
            {
                let [x, y] = system_node_position(kind, ox, oy, width, height);
                let text: SharedString = format!(
                    "{} · 引用{} / 待解析{}",
                    kind.label(),
                    reference_counts[index],
                    unresolved_counts[index]
                )
                .into();
                let style = window.text_style();
                let run = TextRun {
                    len: text.len(),
                    font: style.font(),
                    color: link_color(kind, true).into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let line = window.text_system().shape_line(text, px(10.), &[run], None);
                let _ = line.paint(
                    point(px(x - 170.), px(y - 7.)),
                    px(14.),
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                );
            }
        }
    }
    if let Ok(path) = selected_nodes.build() {
        window.paint_path(path, rgba(0xFFFFFFFF));
    }
    if selected_counts.iter().any(|count| *count > 0) {
        if let Ok(path) = source_points.build() {
            window.paint_path(path, rgba(0x58A6FFFF));
        }
        if let Ok(path) = target_points.build() {
            window.paint_path(path, rgba(0xF2C46FFF));
        }
    }
}
