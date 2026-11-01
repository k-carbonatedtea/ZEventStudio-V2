use crate::model::{map_editor::MapDocumentState, model_preview::ModelDocumentState};
use crate::ui::views::model_preview::wgpu_renderer::{
    compute_cache_key, render_wgpu_scene, types::WgpuCacheKey,
};
use gpui::RenderImage;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

pub struct CompletedFrame {
    key: WgpuCacheKey,
    pub image: Option<Arc<RenderImage>>,
}

struct Request {
    key: WgpuCacheKey,
    scene: ModelDocumentState,
    width: f32,
    height: f32,
}

#[derive(Default)]
struct Mailbox {
    requested: Option<WgpuCacheKey>,
    pending: Option<Request>,
    completed: Option<Arc<CompletedFrame>>,
}

fn worker() -> &'static Arc<(Mutex<Mailbox>, Condvar)> {
    static WORKER: OnceLock<Arc<(Mutex<Mailbox>, Condvar)>> = OnceLock::new();
    WORKER.get_or_init(|| {
        let shared = Arc::new((Mutex::new(Mailbox::default()), Condvar::new()));
        let background = shared.clone();
        std::thread::Builder::new()
            .name("map-viewport".into())
            .spawn(move || loop {
                let (lock, wake) = &*background;
                let request = {
                    let mut mailbox = lock.lock().unwrap();
                    while mailbox.pending.is_none() {
                        mailbox = wake.wait(mailbox).unwrap();
                    }
                    mailbox.pending.take().unwrap()
                };
                let image = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    render_wgpu_scene(&request.scene, request.width, request.height).or_else(|| {
                        crate::ui::views::model_preview::rasterizer::rasterize_scene(
                            &request.scene,
                            request.width,
                            request.height,
                        )
                    })
                }))
                .ok()
                .flatten();
                let frame = Arc::new(CompletedFrame {
                    key: request.key,
                    image,
                });
                lock.lock().unwrap().completed = Some(frame);
            })
            .expect("create map render worker");
        shared
    })
}

fn same_scene(a: &WgpuCacheKey, b: &WgpuCacheKey) -> bool {
    a.geometry_identity == b.geometry_identity
        && a.instance_identity == b.instance_identity
        && a.texture_identity == b.texture_identity
        && a.terrain_identity == b.terrain_identity
        && a.visibility_hash == b.visibility_hash
        && a.model_path == b.model_path
        && a.selected_mesh_idx == b.selected_mesh_idx
        && a.view_w == b.view_w
        && a.view_h == b.view_h
}

/// Keep at most one running render and one replaceable request. UI drawing never
/// waits for GPU completion and never takes the renderer's engine mutex.
pub fn request(
    map: &MapDocumentState,
    width: f32,
    height: f32,
) -> (Option<Arc<CompletedFrame>>, bool) {
    let key = compute_cache_key(
        &map.scene,
        (width.round() as u32).clamp(16, 3840),
        (height.round() as u32).clamp(16, 2160),
    );
    let (lock, wake) = &**worker();
    let mut mailbox = lock.lock().unwrap();
    if mailbox.requested.as_ref() != Some(&key) {
        mailbox.requested = Some(key.clone());
        mailbox.pending = Some(Request {
            key: key.clone(),
            // The renderer only reads the scene. Avoid cloning the full map
            // document and its BYML actor tree while the inspector is open.
            scene: map.scene.clone(),
            width,
            height,
        });
        wake.notify_one();
    }
    let pending = mailbox.completed.as_ref().is_none_or(|f| f.key != key);
    let frame = mailbox
        .completed
        .as_ref()
        .filter(|f| same_scene(&f.key, &key))
        .cloned();
    (frame, pending)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_frame_compatibility_rejects_old_scene_and_size() {
        let scene = crate::model::model_preview::ModelDocumentState::from_bfres(
            "a",
            "a",
            &Default::default(),
        );
        let original = compute_cache_key(&scene, 800, 600);
        let mut key = original.clone();
        key.yaw_bits = 5;
        assert!(same_scene(&original, &key));
        key.instance_identity = 7;
        assert!(!same_scene(&original, &key));
        key = original.clone();
        key.view_w = 400;
        assert!(!same_scene(&original, &key));
    }

    #[test]
    #[ignore = "requires a graphics adapter"]
    fn latest_camera_request_finishes_after_rapid_input() {
        let root = roead::byml::Byml::from_text("Objs: []").unwrap();
        let file = uking_library::map::MapFile::from_bytes(&root.to_binary(roead::Endian::Little))
            .unwrap();
        let mut map = MapDocumentState::empty(file, "async-test", "async-test");
        map.loading = false;
        for i in 0..100 {
            map.scene.camera.yaw = i as f32 * 0.01;
            request(&map, 128., 128.);
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            let (frame, pending) = request(&map, 128., 128.);
            if !pending {
                let frame = frame.expect("completed latest frame");
                assert_eq!(frame.key.yaw_bits, map.scene.camera.yaw.to_bits());
                assert!(frame.image.is_some());
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "latest request was lost"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
