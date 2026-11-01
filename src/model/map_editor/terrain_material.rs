use super::resources::Resources;
use crate::model::model_preview::RgbaTexture;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use uking_library::terrain::TerrainTable;

// TSCB indices describe 88 materials, with five aliases in the 83-layer
// MaterialAlb array. This is the same BOTW mapping used by Project Rito.
const ARRAY_LAYERS: [u32; 88] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 17, 18, 0, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 7, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69,
    70, 71, 0, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82,
];

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
        let resource = resources
            .read("Model/Terrain.Tex1.sbfres")
            .or_else(|| resources.read("Model/Terrain.Tex.sbfres"))
            .ok_or("找不到地形纹理包 Terrain.Tex1.sbfres")?;
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
                    let image = image::imageops::resize(
                        &image,
                        256,
                        256,
                        image::imageops::FilterType::Triangle,
                    );
                    self.images.insert(
                        i as u32,
                        Arc::new(RgbaTexture {
                            width: 256,
                            height: 256,
                            rgba: Arc::new(image.into_raw()),
                        }),
                    );
                }
            }
        }
        if self.images.is_empty() {
            Err("该平台的地形纹理格式尚无法解码，请提供 MaterialAlb_*.png 缓存".into())
        } else {
            Ok(())
        }
    }
    pub fn bake(&mut self, table: &TerrainTable, material: &[u8]) -> RgbaTexture {
        let size = 512usize;
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
                    let image = self.images.entry(layer_id).or_insert_with(|| {
                        for root in &self.roots {
                            if let Ok(img) =
                                image::open(root.join(format!("MaterialAlb_{layer_id}.png")))
                            {
                                let img = img
                                    .resize_exact(256, 256, image::imageops::FilterType::Triangle)
                                    .into_rgba8();
                                return Arc::new(RgbaTexture {
                                    width: 256,
                                    height: 256,
                                    rgba: Arc::new(img.into_raw()),
                                });
                            }
                        }
                        self.missing = true;
                        Arc::new(RgbaTexture {
                            width: 1,
                            height: 1,
                            rgba: Arc::new(vec![110, 124, 84, 255]),
                        })
                    });
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
            let error = reference
                .as_raw()
                .iter()
                .zip(decoded.rgba.iter())
                .map(|(a, b)| (*a as f32 - *b as f32).abs())
                .sum::<f32>()
                / decoded.rgba.len() as f32;
            println!("Terrain layer {layer}: mean absolute channel error {error:.3}");
            assert!(
                error < 3.,
                "layer {layer} differs from decoded reference by {error}"
            );
        }
    }
}
