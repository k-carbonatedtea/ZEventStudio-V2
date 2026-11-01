//! UMii 与 Mii 形象参数、格式转换与 3D 部件动态合成系统

pub mod converter;
pub mod data;
pub mod document;
pub mod parts;
pub mod synthesizer;

#[allow(unused_imports)]
pub use converter::{convert_mii_to_umii, parse_switch_charinfo, parse_wii_mii, parse_wiiu_3ds_mii};
pub use data::UmiiData;
pub use document::{UmiiDocumentState, UmiiEditorSection};
#[allow(unused_imports)]
pub use parts::{
    collect_model_search_dirs, generate_part_targets, locate_part_file, resolve_hair_index,
    UmiiPartTarget, HAIR_FALLBACK_TABLE,
};
#[allow(unused_imports)]
pub use synthesizer::{synthesize_npc_model_sync, ModelPlatform};
