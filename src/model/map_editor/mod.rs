pub mod geometry;
pub mod loader;
pub mod resources;
mod terrain_material;
#[cfg(test)]
mod tests;

use super::model_preview::{ModelDocumentState, ModelInstance};
use roead::byml::Byml;
use std::{collections::HashMap, sync::Arc};
use uking_library::{
    map::{actor_name, MapFile, Transform},
    terrain::HeightField,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ActorVisual {
    pub meshes: Vec<usize>,
    pub resource: String,
    pub has_model: bool,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MapAssets {
    pub actors: HashMap<String, ActorVisual>,
    pub terrain_meshes: Vec<usize>,
    pub background_meshes: Vec<usize>,
    pub water_meshes: Vec<usize>,
    pub heights: Vec<HeightField>,
    pub round_markers: [usize; 2],
    pub context_actors: Vec<Byml>,
    pub context_path: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapInput {
    Search,
    Transform(usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapDocumentState {
    pub file: MapFile,
    pub scene: ModelDocumentState,
    pub assets: Arc<MapAssets>,
    pub selected: Option<usize>,
    pub show_terrain: bool,
    pub show_water: bool,
    pub show_models: bool,
    pub show_tags: bool,
    pub filter: String,
    pub input: Option<MapInput>,
    pub input_text: String,
    pub undo: Vec<Arc<Byml>>,
    pub redo: Vec<Arc<Byml>>,
    pub instance_owners: Arc<Vec<Option<usize>>>,
    pub orbiting: bool,
    pub panning: bool,
    pub dragging: bool,
    pub last_mouse: (f32, f32),
    pub press_mouse: (f32, f32),
    pub drag_changed: bool,
    pub loading: bool,
    pub load_started: bool,
    pub load_error: Option<String>,
    pub page: usize,
}

impl MapDocumentState {
    pub fn empty(file: MapFile, path: &str, title: &str) -> Self {
        let mut scene = ModelDocumentState::from_bfres(path, title, &Default::default());
        scene.is_switch = !file.big_endian;
        scene.show_grid = false;
        scene.show_axes = false;
        Self {
            file,
            scene,
            assets: Arc::new(MapAssets::default()),
            selected: None,
            show_terrain: true,
            show_water: true,
            show_models: true,
            show_tags: false,
            filter: String::new(),
            input: None,
            input_text: String::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            instance_owners: Arc::new(Vec::new()),
            orbiting: false,
            panning: false,
            dragging: false,
            last_mouse: (0., 0.),
            press_mouse: (0., 0.),
            drag_changed: false,
            loading: true,
            load_started: false,
            load_error: None,
            page: 0,
        }
    }

    pub fn rebuild_instances(&mut self) {
        let mut instances = Vec::new();
        let mut owners = Vec::new();
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for (index, actor) in self.file.actors().iter().enumerate() {
            let Some(visual) = self.assets.actors.get(actor_name(actor)) else {
                continue;
            };
            if (visual.has_model && !self.show_models)
                || (!visual.has_model && !self.show_tags && self.selected != Some(index))
            {
                continue;
            }
            let transform = Transform::from_actor(actor);
            let shape = actor
                .as_map()
                .ok()
                .and_then(|m| m.get("!Parameters"))
                .and_then(|p| p.as_map().ok())
                .and_then(|p| p.get("Shape"))
                .and_then(|p| p.as_string().ok())
                .map(|s| s.as_str())
                .unwrap_or("");
            let marker = if !visual.has_model {
                match shape {
                    "Sphere" => Some(self.assets.round_markers[0]),
                    "Cylinder" => Some(self.assets.round_markers[1]),
                    _ => None,
                }
            } else {
                None
            };
            let indices = marker
                .map(|i| vec![i])
                .unwrap_or_else(|| visual.meshes.clone());
            for mesh_index in indices {
                instances.push(ModelInstance {
                    mesh_index,
                    transform,
                    selected: self.selected == Some(index),
                });
                owners.push(Some(index));
                let b = &self.scene.meshes[mesh_index].bounding_box;
                let lo = b.min();
                let hi = b.max();
                for x in [lo.x, hi.x] {
                    for y in [lo.y, hi.y] {
                        for z in [lo.z, hi.z] {
                            let p = transform.point([x, y, z]);
                            for a in 0..3 {
                                min[a] = min[a].min(p[a]);
                                max[a] = max[a].max(p[a]);
                            }
                        }
                    }
                }
            }
        }
        for (&mesh_index, visible) in self
            .assets
            .terrain_meshes
            .iter()
            .map(|i| (i, self.show_terrain))
            .chain(
                self.assets
                    .water_meshes
                    .iter()
                    .map(|i| (i, self.show_water)),
            )
            .chain(
                self.assets
                    .background_meshes
                    .iter()
                    .map(|i| (i, self.show_models)),
            )
        {
            if !visible {
                continue;
            }
            instances.push(ModelInstance {
                mesh_index,
                transform: Transform::default(),
                selected: false,
            });
            owners.push(None);
            let b = &self.scene.meshes[mesh_index].bounding_box;
            let lo = [b.min().x, b.min().y, b.min().z];
            let hi = [b.max().x, b.max().y, b.max().z];
            for a in 0..3 {
                min[a] = min[a].min(lo[a]);
                max[a] = max[a].max(hi[a]);
            }
        }
        for actor in &self.assets.context_actors {
            let Some(visual) = self.assets.actors.get(actor_name(actor)) else {
                continue;
            };
            if !visual.has_model || !self.show_models {
                continue;
            }
            let transform = Transform::from_actor(actor);
            for &mesh_index in &visual.meshes {
                instances.push(ModelInstance {
                    mesh_index,
                    transform,
                    selected: false,
                });
                owners.push(None);
                let b = &self.scene.meshes[mesh_index].bounding_box;
                let lo = b.min();
                let hi = b.max();
                for x in [lo.x, hi.x] {
                    for y in [lo.y, hi.y] {
                        for z in [lo.z, hi.z] {
                            let p = transform.point([x, y, z]);
                            for a in 0..3 {
                                min[a] = min[a].min(p[a]);
                                max[a] = max[a].max(p[a]);
                            }
                        }
                    }
                }
            }
        }
        if min[0].is_finite() {
            self.scene.bounding_box_min = min;
            self.scene.bounding_box_max = max;
        }
        self.scene.total_vertices = instances
            .iter()
            .map(|i| self.scene.meshes[i.mesh_index].positions.len())
            .sum();
        self.scene.total_triangles = instances
            .iter()
            .map(|i| self.scene.meshes[i.mesh_index].indices.len() / 3)
            .sum();
        self.scene.instances = Some(Arc::new(instances));
        self.instance_owners = Arc::new(owners);
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index.filter(|i| *i < self.file.actors().len());
        self.input = None;
        self.rebuild_instances();
    }

    pub fn checkpoint(&mut self) {
        if self.undo.last().is_none_or(|p| p != &self.file.root) {
            self.undo.push(self.file.root.clone());
            if self.undo.len() > 100 {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
    }

    pub fn undo(&mut self) {
        if let Some(root) = self.undo.pop() {
            self.redo.push(self.file.root.clone());
            self.file.root = root;
            self.selected = None;
            self.rebuild_instances();
        }
    }
    pub fn redo(&mut self) {
        if let Some(root) = self.redo.pop() {
            self.undo.push(self.file.root.clone());
            self.file.root = root;
            self.selected = None;
            self.rebuild_instances();
        }
    }
    pub fn revert(&mut self) {
        self.checkpoint();
        self.file.root = self.file.original.clone();
        self.selected = None;
        self.rebuild_instances();
    }
    pub fn change_transform(&mut self, transform: Transform) -> Result<(), String> {
        let index = self.selected.ok_or("请先选择对象")?;
        if Transform::from_actor(&self.file.actors()[index]) == transform {
            return Ok(());
        }
        self.checkpoint();
        self.file.set_transform(index, transform)?;
        self.rebuild_instances();
        Ok(())
    }
    pub fn apply_input(&mut self) -> Result<(), String> {
        if let Some(MapInput::Transform(kind, axis)) = self.input {
            let value: f32 = self
                .input_text
                .trim()
                .parse()
                .map_err(|_| "请输入有效数值")?;
            if !value.is_finite() {
                return Err("请输入有限数值".into());
            }
            let index = self.selected.ok_or("请先选择对象")?;
            let mut t = Transform::from_actor(&self.file.actors()[index]);
            match kind {
                0 => t.translation[axis] = value,
                1 => t.rotation[axis] = value.to_radians(),
                _ => t.scale[axis] = value,
            }
            self.change_transform(t)?;
        }
        self.input = None;
        Ok(())
    }
    pub fn duplicate(&mut self) -> Result<(), String> {
        let index = self.selected.ok_or("请先选择对象")?;
        self.checkpoint();
        let reserved: Vec<_> = self
            .assets
            .context_actors
            .iter()
            .filter_map(uking_library::map::hash_id)
            .collect();
        let index = self.file.duplicate_avoiding(index, &reserved)?;
        self.select(Some(index));
        Ok(())
    }
    pub fn delete(&mut self) -> Result<(), String> {
        let index = self.selected.ok_or("请先选择对象")?;
        self.checkpoint();
        self.file.delete(index)?;
        self.select(None);
        Ok(())
    }
    pub fn snap_to_terrain(&mut self) -> Result<(), String> {
        let index = self.selected.ok_or("请先选择对象")?;
        let mut t = Transform::from_actor(&self.file.actors()[index]);
        let height = self
            .assets
            .heights
            .iter()
            .filter_map(|h| {
                h.height_at(t.translation[0], t.translation[2])
                    .map(|y| (h.tile.size, y))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .ok_or("该位置没有已加载的地形")?
            .1;
        t.translation[1] = height;
        self.change_transform(t)
    }
    pub fn focus_selected(&mut self) {
        if let Some(actor) = self.selected.and_then(|i| self.file.actors().get(i)) {
            let transform = Transform::from_actor(actor);
            let mut min = [f32::INFINITY; 3];
            let mut max = [f32::NEG_INFINITY; 3];
            if let Some(visual) = self.assets.actors.get(actor_name(actor)) {
                for &index in &visual.meshes {
                    let Some(mesh) = self.scene.meshes.get(index) else {
                        continue;
                    };
                    let lo = mesh.bounding_box.min();
                    let hi = mesh.bounding_box.max();
                    for x in [lo.x, hi.x] {
                        for y in [lo.y, hi.y] {
                            for z in [lo.z, hi.z] {
                                let p = transform.point([x, y, z]);
                                for a in 0..3 {
                                    min[a] = min[a].min(p[a]);
                                    max[a] = max[a].max(p[a]);
                                }
                            }
                        }
                    }
                }
            }
            if min.iter().chain(&max).all(|v| v.is_finite()) {
                // Models can have an offset origin and several separated shapes.
                self.scene.camera.fit_bounding_box(min, max);
                self.scene.camera.distance = self.scene.camera.distance.max(8.);
            } else {
                self.scene.camera.target = transform.translation;
                self.scene.camera.distance = 8.;
            }
        }
    }

    /// Nearest actual triangle hit. Local AABB rejection avoids testing every triangle.
    pub fn pick(&self, x: f32, y: f32, width: f32, height: f32) -> Option<usize> {
        let (right, up, forward) = self.scene.camera.compute_basis();
        let tan = (self.scene.camera.fov / 2.).tan();
        let nx = (2. * x / width - 1.) * width / height * tan;
        let ny = (1. - 2. * y / height) * tan;
        let direction = std::array::from_fn(|i| forward[i] + right[i] * nx + up[i] * ny);
        let eye = self.scene.camera.compute_eye();
        let mut nearest = f32::INFINITY;
        let mut hit = None;
        for (instance, owner) in self
            .scene
            .instances
            .as_ref()?
            .iter()
            .zip(self.instance_owners.iter())
        {
            if let Some(distance) = geometry::ray_mesh(
                eye,
                direction,
                instance.transform,
                &self.scene.meshes[instance.mesh_index],
                nearest,
            ) {
                nearest = distance;
                hit = *owner;
            }
        }
        hit
    }
}
