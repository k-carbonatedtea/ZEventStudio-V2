mod display;
pub mod geometry;
pub mod links;
pub mod loader;
pub mod properties;
pub mod resources;
mod terrain_material;
#[cfg(test)]
mod tests;

use super::model_preview::{ModelDocumentState, ModelInstance};
pub use display::MapViewMode;
pub use links::{ActorRef, LinkKindSource, LinkTargetKind, ObjectLink, SystemReference};
pub use properties::{PropertyPart, PropertyPath, PropertyType};
use roead::byml::Byml;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use uking_library::{
    map::{actor_name, MapFile, Transform},
    terrain::HeightField,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ActorVisual {
    pub meshes: Vec<usize>,
    pub resource: String,
    pub profile: String,
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
    pub area_actors: HashSet<String>,
    pub context_actors: Vec<Byml>,
    pub context_path: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapInput {
    Search,
    Transform(usize, usize),
    Property,
    PropertyName,
    NewPropertyName,
    LinkDefinition,
    LinkTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapDocumentState {
    pub file: MapFile,
    pub scene: ModelDocumentState,
    pub assets: Arc<MapAssets>,
    pub selected: Option<usize>,
    pub selected_context: Option<usize>,
    pub view_mode: MapViewMode,
    pub category_filter: Option<LinkTargetKind>,
    pub show_terrain: bool,
    pub show_water: bool,
    pub show_models: bool,
    pub show_tags: bool,
    pub show_hidden_perspective: bool,
    pub show_map_models: bool,
    pub show_context: bool,
    pub show_links: bool,
    pub selected_links_only: bool,
    pub hidden_objects: Arc<HashSet<ActorRef>>,
    pub links: Arc<Vec<ObjectLink>>,
    pub actor_kinds: Arc<HashMap<ActorRef, LinkTargetKind>>,
    pub related_kinds: Arc<HashMap<ActorRef, u8>>,
    pub system_references: Arc<Vec<SystemReference>>,
    link_incoming: Arc<HashMap<ActorRef, Vec<usize>>>,
    link_outgoing: Arc<HashMap<ActorRef, Vec<usize>>>,
    reference_indices: Arc<HashMap<ActorRef, Vec<usize>>>,
    pub selected_link_indices: Arc<Vec<usize>>,
    pub context_objects: Arc<HashSet<ActorRef>>,
    link_cache_source: Option<(Arc<Byml>, Arc<MapAssets>)>,
    pub collapsed_sections: HashSet<String>,
    pub edit_path: PropertyPath,
    pub add_property_parent: PropertyPath,
    pub new_property_name: String,
    pub new_property_type: PropertyType,
    pub new_link_definition: String,
    pub new_link_target: String,
    pub input_error: Option<String>,
    pub outliner_scope: usize,
    pub outliner_width: f32,
    pub inspector_width: f32,
    pub panel_resize: Option<(bool, f32, f32)>,
    pub filter: String,
    pub input: Option<MapInput>,
    pub input_text: String,
    pub undo: Vec<Arc<Byml>>,
    pub redo: Vec<Arc<Byml>>,
    pub instance_owners: Arc<Vec<Option<ActorRef>>>,
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
    fn actor_meshes(&self, actor: &Byml, visual: &ActorVisual) -> Vec<usize> {
        let shape = actor
            .as_map()
            .ok()
            .and_then(|m| m.get("!Parameters"))
            .and_then(|p| p.as_map().ok())
            .and_then(|p| p.get("Shape"))
            .and_then(|p| p.as_string().ok())
            .map(|s| s.as_str())
            .unwrap_or("");
        let is_area = self.assets.area_actors.contains(actor_name(actor));
        match (visual.has_model, is_area, shape) {
            (false, true, "Sphere") => vec![self.assets.round_markers[0]],
            (false, true, "Cylinder") => vec![self.assets.round_markers[1]],
            _ => visual.meshes.clone(),
        }
    }
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
            selected_context: None,
            view_mode: MapViewMode::Context,
            category_filter: None,
            show_terrain: true,
            show_water: true,
            show_models: true,
            show_tags: false,
            show_hidden_perspective: false,
            show_map_models: true,
            show_context: true,
            show_links: true,
            // Keep the initial map view readable; the toolbar can enable all links.
            selected_links_only: true,
            hidden_objects: Arc::new(HashSet::new()),
            links: Arc::new(Vec::new()),
            actor_kinds: Arc::new(HashMap::new()),
            related_kinds: Arc::new(HashMap::new()),
            system_references: Arc::new(Vec::new()),
            link_incoming: Arc::new(HashMap::new()),
            link_outgoing: Arc::new(HashMap::new()),
            reference_indices: Arc::new(HashMap::new()),
            selected_link_indices: Arc::new(Vec::new()),
            context_objects: Arc::new(HashSet::new()),
            link_cache_source: None,
            collapsed_sections: [
                "project-resources".into(),
                "source".into(),
                "display".into(),
            ]
            .into_iter()
            .collect(),
            edit_path: Vec::new(),
            add_property_parent: Vec::new(),
            new_property_name: String::new(),
            new_property_type: PropertyType::Float,
            new_link_definition: "BasicSig".into(),
            new_link_target: String::new(),
            input_error: None,
            outliner_scope: 0,
            outliner_width: 220.,
            inspector_width: 390.,
            panel_resize: None,
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
        self.rebuild_links();
        let mut instances = Vec::new();
        let mut owners = Vec::new();
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for (index, actor) in self.file.actors().iter().enumerate() {
            let Some(visual) = self.assets.actors.get(actor_name(actor)) else {
                continue;
            };
            if !self.object_visible(ActorRef::Current(index)) {
                continue;
            }
            let transform = Transform::from_actor(actor);
            let indices = self.actor_meshes(actor, visual);
            for mesh_index in indices {
                instances.push(ModelInstance {
                    mesh_index,
                    transform,
                    selected: self.selected == Some(index),
                });
                owners.push(Some(ActorRef::Current(index)));
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
                    .map(|i| (i, self.show_map_models)),
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
        for (index, actor) in self.assets.context_actors.iter().enumerate() {
            let Some(visual) = self.assets.actors.get(actor_name(actor)) else {
                continue;
            };
            if !self.object_visible(ActorRef::Context(index)) {
                continue;
            }
            let transform = Transform::from_actor(actor);
            for mesh_index in self.actor_meshes(actor, visual) {
                instances.push(ModelInstance {
                    mesh_index,
                    transform,
                    selected: self.selected_context == Some(index),
                });
                owners.push(Some(ActorRef::Context(index)));
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
        self.selected_context = None;
        self.selected = index.filter(|i| *i < self.file.actors().len());
        self.input = None;
        self.input_error = None;
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
            self.selected_context = None;
            self.input = None;
            self.input_error = None;
            self.hidden_objects = Arc::new(HashSet::new());
            self.refresh_after_history();
        }
    }
    pub fn redo(&mut self) {
        if let Some(root) = self.redo.pop() {
            self.undo.push(self.file.root.clone());
            self.file.root = root;
            self.selected = None;
            self.selected_context = None;
            self.input = None;
            self.input_error = None;
            self.hidden_objects = Arc::new(HashSet::new());
            self.refresh_after_history();
        }
    }
    pub fn revert(&mut self) {
        self.checkpoint();
        self.file.root = self.file.original.clone();
        self.selected = None;
        self.selected_context = None;
        self.input = None;
        self.input_error = None;
        self.hidden_objects = Arc::new(HashSet::new());
        self.refresh_after_history();
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
        let result = self.apply_input_inner();
        self.input_error = result.as_ref().err().cloned();
        result
    }
    fn apply_input_inner(&mut self) -> Result<(), String> {
        if self.apply_property_input()? {
            self.input = None;
            self.input_error = None;
            return Ok(());
        }
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
        self.input_error = None;
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
        if uking_library::map::hash_id(&self.file.actors()[index])
            .is_some_and(|id| self.has_context_reference(id))
        {
            return Err("配套地图中有链接指向此对象，无法单独删除".into());
        }
        self.checkpoint();
        self.file.delete(index)?;
        self.hidden_objects = Arc::new(HashSet::new());
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
        if let Some(actor) = self.selected_actor() {
            let transform = Transform::from_actor(actor);
            let mut min = [f32::INFINITY; 3];
            let mut max = [f32::NEG_INFINITY; 3];
            if let Some(visual) = self.assets.actors.get(actor_name(actor)) {
                for index in self.actor_meshes(actor, visual) {
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
    pub fn pick_with_camera(
        &self,
        camera: &super::model_preview::OrbitCamera,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<ActorRef> {
        let (right, up, forward) = camera.compute_basis();
        let tan = (camera.fov / 2.).tan();
        let nx = (2. * x / width - 1.) * width / height * tan;
        let ny = (1. - 2. * y / height) * tan;
        let direction = std::array::from_fn(|i| forward[i] + right[i] * nx + up[i] * ny);
        let eye = camera.compute_eye();
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
