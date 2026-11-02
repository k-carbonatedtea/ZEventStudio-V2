use super::{geometry, MapDocumentState};
use crate::model::model_preview::OrbitCamera;

impl MapDocumentState {
    pub fn asset_spawn_position(&self) -> [f32; 3] {
        let mut position = self.scene.camera.target;
        if let Some(height) = self
            .assets
            .heights
            .iter()
            .filter_map(|field| {
                field
                    .height_at(position[0], position[2])
                    .map(|y| (field.tile.size, y))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, y)| y)
        {
            position[1] = height;
        }
        position
    }

    /// Drop on the closest rendered surface. Logical editor markers are not
    /// collision surfaces. Empty space falls back to the camera target plane.
    pub fn asset_drop_position(
        &self,
        camera: &OrbitCamera,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<[f32; 3]> {
        if ![x, y, width, height].iter().all(|v| v.is_finite()) || width <= 0. || height <= 0. {
            return None;
        }
        let (right, up, forward) = camera.compute_basis();
        let tan = (camera.fov / 2.).tan();
        let nx = (2. * x / width - 1.) * width / height * tan;
        let ny = (1. - 2. * y / height) * tan;
        let direction: [f32; 3] = std::array::from_fn(|i| forward[i] + right[i] * nx + up[i] * ny);
        let eye = camera.compute_eye();
        let mut nearest = f32::INFINITY;
        if let Some(instances) = &self.scene.instances {
            for (instance, owner) in instances.iter().zip(self.instance_owners.iter()) {
                if owner.is_some_and(|reference| {
                    self.actor(reference)
                        .and_then(|actor| {
                            self.assets
                                .actors
                                .get(uking_library::map::actor_name(actor))
                        })
                        .is_none_or(|visual| !visual.has_model)
                }) {
                    continue;
                }
                let Some(mesh) = self.scene.meshes.get(instance.mesh_index) else {
                    continue;
                };
                if let Some(distance) =
                    geometry::ray_mesh(eye, direction, instance.transform, mesh, nearest)
                {
                    nearest = distance;
                }
            }
        }
        let distance = if nearest.is_finite() {
            nearest
        } else if direction[1].abs() > 1e-6 {
            let distance = (camera.target[1] - eye[1]) / direction[1];
            if distance > 0. {
                distance
            } else {
                camera.distance.max(1.)
            }
        } else {
            camera.distance.max(1.)
        };
        let result = std::array::from_fn(|i| eye[i] + direction[i] * distance);
        result.iter().all(|v| v.is_finite()).then_some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roead::byml::Byml;
    use std::sync::Arc;
    #[test]
    fn drop_uses_nearest_surface_and_rejects_invalid_viewport() {
        let root = Byml::from_text("Objs: []").unwrap();
        let file = uking_library::map::MapFile::from_bytes(&root.to_binary(roead::Endian::Little))
            .unwrap();
        let mut doc = MapDocumentState::empty(file, "placement", "placement");
        let index = super::super::loader::push_mesh(&mut doc.scene, geometry::cube("floor"));
        doc.scene.instances = Some(Arc::new(vec![crate::model::model_preview::ModelInstance {
            mesh_index: index,
            transform: Default::default(),
            selected: false,
        }]));
        doc.instance_owners = Arc::new(vec![None]);
        doc.scene.camera.target = [0., 0., 0.];
        doc.scene.camera.distance = 10.;
        let position = doc
            .asset_drop_position(&doc.scene.camera, 400., 300., 800., 600.)
            .unwrap();
        assert!(position.iter().any(|v| (v.abs() - 1.).abs() < 0.001));
        assert!(position.iter().all(|v| v.abs() <= 1.001));
        assert!(doc
            .asset_drop_position(&doc.scene.camera, 0., 0., 0., 600.)
            .is_none());
    }
}
