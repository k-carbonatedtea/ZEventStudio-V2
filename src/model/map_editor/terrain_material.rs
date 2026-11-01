use super::resources::Resources;
use crate::model::model_preview::{types::TerrainRenderData, RgbaTexture};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use uking_library::terrain::TerrainTable;

// TSCB indices describe 88 materials, with five aliases in the 83-layer
// MaterialAlb array. This is the same BOTW mapping used by Project Rito.
const ARRAY_LAYERS: [u32; 88] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 17, 18, 0, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 7, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69,
    70, 71, 0, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82,
];

type LayerImages = HashMap<u32, Arc<RgbaTexture>>;
// Keep one decoded source across map tabs; source metadata invalidates edits.
// Both this entry and loaded scenes share the same image allocations.
static LAST_ARRAY: Mutex<Option<(String, u64, SystemTime, LayerImages)>> = Mutex::new(None);

pub struct TerrainMaterials {
    roots: Vec<PathBuf>,
    images: HashMap<u32, Arc<RgbaTexture>>,
    pub missing: bool,
}
impl TerrainMaterials {
    pub fn new(project: Option<&str>) -> Self {
        let mut roots = Vec::new();
        if let Some(path) = std::env::var_os("ZEVENT_TERRAIN_CACHE") {
            roots.push(PathBuf::from(path));
        }
        if let Some(project) = project {
            roots.push(PathBuf::from(project).join("Terrain/Materials"));
        }
        if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd.join(".cache/map_terrain"));
            roots.push(cwd.join("assets/map/terrain"));
        }
        if let Ok(exe) = std::env::current_exe() {
            for parent in exe.ancestors().skip(1).take(4) {
                roots.push(parent.join(".cache/map_terrain"));
            }
        }
        Self {
            roots,
            images: HashMap::new(),
            missing: false,
        }
    }
    /// Decode the game's texture array directly; PNGs provide a fallback.
    pub fn load_game_array(&mut self, resources: &mut Resources) -> Result<(), String> {
        if !self.images.is_empty() {
            return Ok(());
        }
        let resource = resources
            .read("Model/Terrain.Tex1.sbfres")
            .or_else(|| resources.read("Model/Terrain.Tex.sbfres"))
            .ok_or("找不到地形纹理包 Terrain.Tex1.sbfres")?;
        let outer = resource
            .path
            .trim_start_matches("SARC:")
            .split("//")
            .next()
            .unwrap_or("");
        let stamp = std::fs::metadata(outer)
            .ok()
            .and_then(|m| m.modified().ok().map(|t| (m.len(), t)));
        if let Some((length, modified)) = stamp {
            if let Ok(cache) = LAST_ARRAY.lock() {
                if let Some((path, len, time, images)) = &*cache {
                    if path == &resource.path && *len == length && *time == modified {
                        self.images = images.clone();
                        return Ok(());
                    }
                }
            }
        }
        let bytes = uking_library::decompress(&resource.bytes)?;
        let bfres = rbfres_library::BfresFile::from_bytes(&bytes).map_err(|e| e.to_string())?;
        for (_, external) in bfres.external_files.iter() {
            if !external.data.starts_with(b"BNTX") {
                continue;
            }
            let bntx = rbfres_library::texture::BntxFile::from_bytes(&external.data)
                .map_err(|e| e.to_string())?;
            for mut texture in bntx
                .textures
                .into_iter()
                .filter(|t| t.name == "MaterialAlb")
            {
                let data = std::mem::take(&mut texture.data);
                let count = texture.array_length as usize;
                if count == 0 || data.len() % count != 0 {
                    return Err("地形纹理数组长度无效".into());
                }
                let stride = data.len() / count;
                for (i, layer) in data.chunks_exact(stride).enumerate() {
                    texture.data = layer.to_vec();
                    let rgba = texture.decode_rgba();
                    let image = image::RgbaImage::from_raw(texture.width, texture.height, rgba)
                        .ok_or("无法解码地形纹理层")?;
                    self.images.insert(
                        i as u32,
                        Arc::new(RgbaTexture {
                            width: image.width(),
                            height: image.height(),
                            rgba: Arc::new(image.into_raw()),
                        }),
                    );
                }
            }
        }
        if self.images.is_empty() {
            Err("该平台的地形纹理格式尚无法解码，请提供 MaterialAlb_*.png 缓存".into())
        } else {
            if let Some((length, modified)) = stamp {
                if let Ok(mut cache) = LAST_ARRAY.lock() {
                    *cache = Some((resource.path, length, modified, self.images.clone()));
                }
            }
            Ok(())
        }
    }
    pub fn bake(&mut self, table: &TerrainTable, material: &[u8]) -> RgbaTexture {
        // Native materials render on the GPU. Avoid expensive, blurry CPU baking
        // on the normal path; this thumbnail remains usable without a GPU.
        let size = 128usize;
        let mut rgba = vec![0; size * size * 4];
        for y in 0..size {
            for x in 0..size {
                let sample = ((y * 255 / (size - 1)) * 256 + x * 255 / (size - 1)) * 4;
                let blend = material[sample + 2] as f32 / 255.;
                let mut colors = [[110, 124, 84, 255]; 2];
                for layer in 0..2 {
                    if (layer == 0 && blend == 1.) || (layer == 1 && blend == 0.) {
                        continue;
                    }
                    let index = material[sample + layer] as usize;
                    let material_id =
                        table.material_layers.get(index).copied().unwrap_or(0) as usize;
                    let layer_id = ARRAY_LAYERS.get(material_id).copied().unwrap_or(0);
                    let image = self.image(layer_id);
                    let uv = table.material_uvs.get(index).copied().unwrap_or([0.1, 0.1]);
                    colors[layer] = image.sample_bilinear(
                        x as f32 / (size - 1) as f32 * 100. * uv[0],
                        y as f32 / (size - 1) as f32 * 100. * uv[1],
                    );
                }
                for c in 0..3 {
                    rgba[(y * size + x) * 4 + c] =
                        (colors[0][c] as f32 * (1. - blend) + colors[1][c] as f32 * blend) as u8;
                }
                rgba[(y * size + x) * 4 + 3] = 255;
            }
        }
        RgbaTexture {
            width: size as u32,
            height: size as u32,
            rgba: Arc::new(rgba),
        }
    }

    pub fn layer(&self, index: u32) -> Option<Arc<RgbaTexture>> {
        self.images.get(&index).cloned()
    }

    fn image(&mut self, layer_id: u32) -> Arc<RgbaTexture> {
        if let Some(image) = self.images.get(&layer_id) {
            return image.clone();
        }
        let image = self
            .roots
            .iter()
            .find_map(|root| {
                let image = image::open(root.join(format!("MaterialAlb_{layer_id}.png")))
                    .ok()?
                    .into_rgba8();
                Some(Arc::new(RgbaTexture {
                    width: image.width(),
                    height: image.height(),
                    rgba: Arc::new(image.into_raw()),
                }))
            })
            .unwrap_or_else(|| {
                self.missing = true;
                Arc::new(RgbaTexture {
                    width: 1,
                    height: 1,
                    rgba: Arc::new(vec![110, 124, 84, 255]),
                })
            });
        self.images.insert(layer_id, image.clone());
        image
    }

    pub fn render_data(&mut self, table: &TerrainTable) -> TerrainRenderData {
        let material_params = (0..256)
            .map(|index| {
                let material = table.material_layers.get(index).copied().unwrap_or(0) as usize;
                let layer = ARRAY_LAYERS.get(material).copied().unwrap_or(0);
                let uv = table.material_uvs.get(index).copied().unwrap_or([0.1; 2]);
                [100. * uv[0], 100. * uv[1], layer as f32, 0.]
            })
            .collect();
        TerrainRenderData {
            layers: (0..83).map(|i| self.image(i)).collect(),
            material_params,
            tiles: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_materials_keep_detail_and_map_aliases_without_baking() {
        let mut materials = TerrainMaterials::new(None);
        let image = Arc::new(RgbaTexture {
            width: 1024,
            height: 1024,
            rgba: Arc::new(vec![7; 1024 * 1024 * 4]),
        });
        for i in 0..83 {
            materials.images.insert(i, image.clone());
        }
        let table = TerrainTable {
            tiles: vec![],
            material_layers: vec![29, 63],
            material_uvs: vec![[0.05, 0.16], [0.25, 0.04]],
        };
        let data = materials.render_data(&table);
        assert_eq!(data.layers[17].width, 1024);
        assert!(Arc::ptr_eq(&data.layers[17], &image));
        assert_eq!(data.material_params[0], [5., 16., 17., 0.]);
        assert_eq!(data.material_params[1], [25., 4., 7., 0.]);
        assert_eq!(data.material_params.len(), 256);
    }

    #[test]
    #[ignore = "requires configured game dump and local reference PNG cache"]
    fn terrain_array_matches_reference_images() {
        let settings = crate::model::settings::GamePathSettings::load_from_disk();
        let mut resources = Resources::new("F-5_Dynamic.smubin", None, &settings, false);
        let mut materials = TerrainMaterials::new(None);
        materials.load_game_array(&mut resources).unwrap();
        assert_eq!(materials.images.len(), 83);
        for layer in [0, 17, 29, 60, 82] {
            let reference = image::open(format!(".cache/map_terrain/MaterialAlb_{layer}.png"))
                .unwrap()
                .resize_exact(256, 256, image::imageops::FilterType::Triangle)
                .into_rgba8();
            let decoded = &materials.images[&layer];
            assert!(
                decoded.width >= 1024,
                "native terrain detail was downsampled"
            );
            let decoded_image = image::RgbaImage::from_raw(
                decoded.width,
                decoded.height,
                decoded.rgba.as_ref().clone(),
            )
            .unwrap();
            let decoded_image = image::imageops::resize(
                &decoded_image,
                256,
                256,
                image::imageops::FilterType::Triangle,
            );
            let error = reference
                .as_raw()
                .iter()
                .zip(decoded_image.as_raw().iter())
                .map(|(a, b)| (*a as f32 - *b as f32).abs())
                .sum::<f32>()
                / decoded_image.as_raw().len() as f32;
            println!("Terrain layer {layer}: mean absolute channel error {error:.3}");
            assert!(
                error < 3.,
                "layer {layer} differs from decoded reference by {error}"
            );
        }
    }
}
