use rbfres_library::{ExtractedMesh, Vec2, Vec3};
use uking_library::{
    map::Transform,
    terrain::{HeightField, SIDE},
};

pub fn cube(name: &str) -> ExtractedMesh {
    let positions = [
        [-1., -1., -1.],
        [1., -1., -1.],
        [1., 1., -1.],
        [-1., 1., -1.],
        [-1., -1., 1.],
        [1., -1., 1.],
        [1., 1., 1.],
        [-1., 1., 1.],
    ]
    .into_iter()
    .map(|p| Vec3::new(p[0], p[1], p[2]))
    .collect::<Vec<_>>();
    let bounding_box = ExtractedMesh::compute_bounding_box(&positions);
    ExtractedMesh {
        name: name.into(),
        normals: positions.iter().map(|v| v.normalize()).collect(),
        uvs: vec![Vec2::new(0., 0.); 8],
        positions,
        indices: vec![
            0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 3, 7, 6, 3, 6, 2, 0, 4, 7, 0, 7,
            3, 1, 2, 6, 1, 6, 5,
        ],
        bounding_box,
        material_name: Some(name.into()),
        ..Default::default()
    }
}

/// Rito-compatible cube atlas layout. Each face occupies one 1/4 x 1/2 cell
/// in the shared 512x256 marker textures.
pub fn rito_cube(name: &str, size: f32) -> ExtractedMesh {
    let vertices = [
        ([-1., 1., 1.], [-1., 0., 0.], [0.250018, 0.99999]),
        ([-1., -1., -1.], [-1., 0., 0.], [0.00016, 0.500273]),
        ([-1., -1., 1.], [-1., 0., 0.], [0.250018, 0.500273]),
        ([-1., 1., -1.], [0., 0., -1.], [0.500048, 1.]),
        ([1., -1., -1.], [0., 0., -1.], [0.250073, 0.500051]),
        ([-1., -1., -1.], [0., 0., -1.], [0.500048, 0.500051]),
        ([1., 1., -1.], [1., 0., 0.], [0.749886, 0.999726]),
        ([1., -1., 1.], [1., 0., 0.], [0.500027, 0.500009]),
        ([1., -1., -1.], [1., 0., 0.], [0.749886, 0.500009]),
        ([1., 1., 1.], [0., 0., 1.], [0.999994, 1.]),
        ([-1., -1., 1.], [0., 0., 1.], [0.750019, 0.500051]),
        ([1., -1., 1.], [0., 0., 1.], [0.999994, 0.500051]),
        ([1., -1., -1.], [0., -1., 0.], [0.250004, 0.000051]),
        ([-1., -1., 1.], [0., -1., 0.], [0.499979, 0.500001]),
        ([-1., -1., -1.], [0., -1., 0.], [0.250004, 0.500001]),
        ([-1., 1., -1.], [0., 1., 0.], [0.000026, 0.000048]),
        ([1., 1., 1.], [0., 1., 0.], [0.250001, 0.499998]),
        ([1., 1., -1.], [0., 1., 0.], [0.000026, 0.499998]),
        ([-1., 1., -1.], [-1., 0., 0.], [0.00016, 0.99999]),
        ([1., 1., -1.], [0., 0., -1.], [0.250073, 1.]),
        ([1., 1., 1.], [1., 0., 0.], [0.500027, 0.999726]),
        ([-1., 1., 1.], [0., 0., 1.], [0.750019, 1.]),
        ([1., -1., 1.], [0., -1., 0.], [0.499979, 0.000051]),
        ([-1., 1., 1.], [0., 1., 0.], [0.250001, 0.000048]),
    ];
    let positions = vertices
        .iter()
        .map(|(p, _, _)| Vec3::new(p[0] * size, p[1] * size, p[2] * size))
        .collect::<Vec<_>>();
    let normals = vertices
        .iter()
        .map(|(_, n, _)| Vec3::new(n[0], n[1], n[2]))
        .collect::<Vec<_>>();
    let uvs = vertices
        .iter()
        .map(|(_, _, uv)| Vec2::new(uv[0], uv[1]))
        .collect::<Vec<_>>();
    let indices = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0, 18, 1, 3, 19, 4,
        6, 20, 7, 9, 21, 10, 12, 22, 13, 15, 23, 16,
    ];
    let bounding_box = ExtractedMesh::compute_bounding_box(&positions);
    ExtractedMesh {
        name: name.into(),
        normals,
        uvs,
        positions,
        indices,
        bounding_box,
        material_name: Some(name.into()),
        ..Default::default()
    }
}

pub fn round_marker(sphere: bool) -> ExtractedMesh {
    let mut mesh = ExtractedMesh {
        name: if sphere {
            "Area sphere"
        } else {
            "Area cylinder"
        }
        .into(),
        ..Default::default()
    };
    let rows = if sphere { 16 } else { 1 };
    let columns = 32;
    for row in 0..=rows {
        let angle = row as f32 / rows as f32 * std::f32::consts::PI;
        let (radius, y) = if sphere {
            (angle.sin(), angle.cos())
        } else {
            (1., 1. - 2. * row as f32)
        };
        for col in 0..=columns {
            let theta = col as f32 / columns as f32 * std::f32::consts::TAU;
            let p = Vec3::new(radius * theta.cos(), y, radius * theta.sin());
            mesh.positions.push(p);
            mesh.normals.push(if sphere {
                p.normalize()
            } else {
                Vec3::new(theta.cos(), 0., theta.sin())
            });
            mesh.uvs.push(Vec2::new(
                col as f32 / columns as f32,
                row as f32 / rows as f32,
            ));
        }
    }
    for row in 0..rows {
        for col in 0..columns {
            let a = (row * (columns + 1) + col) as u32;
            let b = a + columns as u32 + 1;
            mesh.indices.extend([a, b, b + 1, b + 1, a + 1, a]);
        }
    }
    if !sphere {
        for row in [0, 1] {
            for col in 1..columns - 1 {
                let a = (row * (columns + 1)) as u32;
                mesh.indices.extend([a, a + col as u32, a + col as u32 + 1]);
            }
        }
    }
    mesh.bounding_box = ExtractedMesh::compute_bounding_box(&mesh.positions);
    mesh
}

/// Trim coarse tiles where a finer tile exists, and keep the requested section.
pub fn clip_tile(
    mesh: &mut ExtractedMesh,
    tile: &uking_library::terrain::TerrainTile,
    tiles: &[&uking_library::terrain::TerrainTile],
    center: [f32; 2],
) {
    let mut indices = Vec::new();
    for tri in mesh.indices.chunks_exact(3) {
        let (mut x, mut z) = (0., 0.);
        for &i in tri {
            x += mesh.positions[i as usize].x / 3.;
            z += mesh.positions[i as usize].z / 3.;
        }
        if (x - center[0]).abs() > 500. || (z - center[1]).abs() > 500. {
            continue;
        }
        if tiles.iter().any(|other| {
            other.size < tile.size
                && (x - other.center[0]).abs() < other.size / 2.
                && (z - other.center[1]).abs() < other.size / 2.
        }) {
            continue;
        }
        indices.extend_from_slice(tri);
    }
    mesh.indices = indices;
    let used: Vec<_> = mesh
        .indices
        .iter()
        .map(|&i| mesh.positions[i as usize])
        .collect();
    if !used.is_empty() {
        mesh.bounding_box = ExtractedMesh::compute_bounding_box(&used);
    }
}

pub fn terrain_mesh(field: &HeightField, water: Option<&[u8]>) -> Result<ExtractedMesh, String> {
    let side = if water.is_some() { 64 } else { SIDE };
    let last = side - 1;
    if water.is_some_and(|w| w.len() != side * side * 8) {
        return Err("水体高度图长度无效".into());
    }
    let mut mesh = ExtractedMesh {
        name: format!(
            "{}{}",
            field.tile.name,
            if water.is_some() {
                " water"
            } else {
                " terrain"
            }
        ),
        ..Default::default()
    };
    let step = field.tile.size / last as f32;
    for z in 0..side {
        for x in 0..side {
            let i = z * side + x;
            let y = water
                .map(|w| {
                    u16::from_le_bytes([w[i * 8], w[i * 8 + 1]]) as f32
                        * uking_library::terrain::HEIGHT_SCALE
                })
                .unwrap_or(field.heights[i]);
            mesh.positions.push(Vec3::new(
                field.tile.center[0] + (x as f32 / last as f32 - 0.5) * field.tile.size,
                y,
                field.tile.center[1] + (z as f32 / last as f32 - 0.5) * field.tile.size,
            ));
            let dx = field.heights[z * SIDE + (x + 1).min(255)]
                - field.heights[z * SIDE + x.saturating_sub(1)];
            let dz = field.heights[(z + 1).min(255) * SIDE + x]
                - field.heights[z.saturating_sub(1) * SIDE + x];
            mesh.normals.push(if water.is_some() {
                Vec3::new(0., 1., 0.)
            } else {
                Vec3::new(-dx, 2. * step, -dz).normalize()
            });
            mesh.uvs
                .push(Vec2::new(x as f32 / last as f32, z as f32 / last as f32));
        }
    }
    for z in 0..last {
        for x in 0..last {
            let a = (z * side + x) as u32;
            let b = a + side as u32;
            for triangle in [[a, b, b + 1], [b + 1, a + 1, a]] {
                if let Some(w) = water {
                    // 0xFFFF/unknown material denotes empty water. Sea-level water
                    // can have height zero and must remain visible.
                    if triangle.iter().any(|&i| w[i as usize * 8 + 7] > 7) {
                        continue;
                    }
                }
                mesh.indices.extend(triangle);
            }
        }
    }
    mesh.bounding_box = ExtractedMesh::compute_bounding_box(&mesh.positions);
    Ok(mesh)
}

/// Empty water samples can form a plane beneath the entire map. Omit fully
/// buried faces; keep intersecting faces so depth testing forms the shoreline.
pub fn clip_buried_water(mesh: &mut ExtractedMesh, heights: &[HeightField]) {
    mesh.indices = mesh
        .indices
        .chunks_exact(3)
        .filter(|tri| {
            tri.iter().any(|&i| {
                let p = mesh.positions[i as usize];
                heights
                    .iter()
                    .filter_map(|h| h.height_at(p.x, p.z).map(|y| (h.tile.size, y)))
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .is_some_and(|(_, y)| p.y >= y - 0.05)
            })
        })
        .flatten()
        .copied()
        .collect();
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn ray_mesh(
    origin: [f32; 3],
    direction: [f32; 3],
    transform: Transform,
    mesh: &ExtractedMesh,
    limit: f32,
) -> Option<f32> {
    if transform.scale.iter().any(|s| s.abs() < 1e-8) {
        return None;
    }
    let m = transform.matrix();
    let delta = sub(origin, transform.translation);
    let local = |v: [f32; 3]| {
        std::array::from_fn(|i| {
            dot(v, [m[i][0], m[i][1], m[i][2]]) / (transform.scale[i] * transform.scale[i])
        })
    };
    let o: [f32; 3] = local(delta);
    let d: [f32; 3] = local(direction);
    let lo = mesh.bounding_box.min();
    let hi = mesh.bounding_box.max();
    let lo = [lo.x, lo.y, lo.z];
    let hi = [hi.x, hi.y, hi.z];
    let mut near: f32 = 0.;
    let mut far = limit;
    for a in 0..3 {
        if d[a].abs() < 1e-10 {
            if o[a] < lo[a] || o[a] > hi[a] {
                return None;
            }
        } else {
            let t1 = (lo[a] - o[a]) / d[a];
            let t2 = (hi[a] - o[a]) / d[a];
            near = near.max(t1.min(t2));
            far = far.min(t1.max(t2));
            if near > far {
                return None;
            }
        }
    }
    let mut closest = limit;
    for tri in mesh.indices.chunks_exact(3) {
        let p = |i: u32| mesh.positions.get(i as usize).map(|v| [v.x, v.y, v.z]);
        let (Some(a), Some(b), Some(c)) = (p(tri[0]), p(tri[1]), p(tri[2])) else {
            continue;
        };
        let e1 = sub(b, a);
        let e2 = sub(c, a);
        let h = cross(d, e2);
        let det = dot(e1, h);
        let face_det = if transform.scale.iter().product::<f32>() < 0. {
            -det
        } else {
            det
        };
        if det.abs() < 1e-9
            || mesh.display_face == 3
            || (mesh.display_face == 1 && face_det < 0.)
            || (mesh.display_face == 2 && face_det > 0.)
        {
            continue;
        }
        let s = sub(o, a);
        let u = dot(s, h) / det;
        if !(0. ..=1.).contains(&u) {
            continue;
        }
        let q = cross(s, e1);
        let v = dot(d, q) / det;
        if v < 0. || u + v > 1. {
            continue;
        }
        let t = dot(e2, q) / det;
        if t > 0. && t < closest {
            closest = t;
        }
    }
    (closest < limit).then_some(closest)
}
