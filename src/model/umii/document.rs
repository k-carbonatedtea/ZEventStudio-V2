//! UMii 交互式编辑器文档状态、分区导航与随机形象算法

use serde::{Deserialize, Serialize};
use super::data::UmiiData;
use super::synthesizer::{synthesize_npc_model_sync, ModelPlatform};
use crate::model::model_preview::ModelDocumentState;
use std::path::PathBuf;

/// UMii 编辑器分栏分区枚举 (与经典 Mii Maker / MyAvatarEditor 规范对齐)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UmiiEditorSection {
    #[default]
    Basic,
    Face,
    Hair,
    Eyebrows,
    Eyes,
    Nose,
    Mouth,
    Beard,
    Accessories,
}

impl UmiiEditorSection {
    pub fn title_key(&self) -> &'static str {
        match self {
            Self::Basic => "umii.section_basic",
            Self::Face => "umii.section_face",
            Self::Hair => "umii.section_hair",
            Self::Eyebrows => "umii.section_eyebrows",
            Self::Eyes => "umii.section_eyes",
            Self::Nose => "umii.section_nose",
            Self::Mouth => "umii.section_mouth",
            Self::Beard => "umii.section_beard",
            Self::Accessories => "umii.section_accessories",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Basic => "👤",
            Self::Face => "🧏",
            Self::Hair => "💇",
            Self::Eyebrows => "🤨",
            Self::Eyes => "👁️",
            Self::Nose => "👃",
            Self::Mouth => "👄",
            Self::Beard => "🧔",
            Self::Accessories => "👓",
        }
    }
}

/// UMii 编辑器活跃文档状态
#[derive(Debug, Clone, PartialEq)]
pub struct UmiiDocumentState {
    pub file_path: String,
    pub raw_name: String,
    pub data: UmiiData,
    pub original_data: UmiiData,
    pub active_section: UmiiEditorSection,
    pub target_platform: ModelPlatform,
    pub synthesized_model: Option<ModelDocumentState>,
    pub is_synthesizing: bool,
    pub last_error: Option<String>,
}

impl UmiiDocumentState {
    /// 新建文档状态
    pub fn new(file_path: String, raw_name: String, data: UmiiData) -> Self {
        Self {
            file_path,
            raw_name,
            original_data: data.clone(),
            data,
            active_section: UmiiEditorSection::Basic,
            target_platform: ModelPlatform::Switch,
            synthesized_model: None,
            is_synthesizing: false,
            last_error: None,
        }
    }

    /// 检查当前参数相较于初始加载是否发生实质修改
    pub fn is_modified(&self) -> bool {
        self.data != self.original_data
    }

    /// 标记数据已成功保存 (重置原始快照)
    pub fn mark_saved(&mut self) {
        self.original_data = self.data.clone();
    }

    /// 恢复为原始快照
    pub fn reset_to_original(&mut self) {
        self.data = self.original_data.clone();
    }

    /// 重新根据当前参数合成 3D 模型
    pub fn synthesize_model(&mut self, search_dirs: &[PathBuf]) -> Result<(), String> {
        self.is_synthesizing = true;
        match synthesize_npc_model_sync(&self.data, self.target_platform, Some(search_dirs), None) {
            Ok(doc) => {
                self.synthesized_model = Some(doc);
                self.is_synthesizing = false;
                self.last_error = None;
                Ok(())
            }
            Err(e) => {
                self.is_synthesizing = false;
                self.last_error = Some(e.clone());
                Err(e)
            }
        }
    }

    /// 伪随机生成一个和谐美观的 NPC 形象
    pub fn randomize(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(123456789);
        let mut rng = SimpleRng::new(seed as u64);

        self.data.hair_type = rng.gen_range(0, 131) as i32;
        self.data.hair_color = rng.gen_range(0, 7) as i32;
        self.data.hair_flip = rng.gen_range(0, 1) == 1;

        self.data.shape_jaw = rng.gen_range(0, 11) as i32;
        self.data.shape_skin_color = rng.gen_range(0, 10) as i32;
        self.data.shape_wrinkle = if rng.gen_range(0, 3) == 0 { rng.gen_range(0, 11) as i32 } else { 0 };
        self.data.shape_make = if rng.gen_range(0, 3) == 0 { rng.gen_range(0, 11) as i32 } else { 0 };

        self.data.eye_type = rng.gen_range(0, 47) as i32;
        self.data.eye_color = rng.gen_range(0, 5) as i32;
        self.data.eye_scale = rng.gen_range(3, 5) as f32;
        self.data.eye_trans_u = rng.gen_range(1, 4) as f32;
        self.data.eye_trans_v = rng.gen_range(8, 14) as f32;

        self.data.eyebrow_type = rng.gen_range(0, 23) as i32;
        self.data.eyebrow_color = self.data.hair_color;
        self.data.eyebrow_scale = rng.gen_range(2, 5) as f32;
        self.data.eyebrow_trans_u = rng.gen_range(1, 4) as f32;
        self.data.eyebrow_trans_v = rng.gen_range(2, 6) as f32;

        self.data.nose_type = rng.gen_range(0, 17) as i32;
        self.data.nose_scale = rng.gen_range(3, 5) as f32;
        self.data.nose_trans_v = rng.gen_range(7, 11) as f32;

        self.data.mouth_type = rng.gen_range(0, 23) as i32;
        self.data.mouth_color = rng.gen_range(0, 4) as i32;
        self.data.mouth_scale = rng.gen_range(3, 5) as f32;
        self.data.mouth_trans_v = rng.gen_range(13, 17) as f32;

        self.data.personal_fav_color = rng.gen_range(0, 11) as i32;
        self.data.body_height = rng.gen_range(20, 110) as i32;
        self.data.body_weight = rng.gen_range(20, 110) as i32;

        // 小概率带有眼镜或胡须
        self.data.glass_type = if rng.gen_range(0, 4) == 0 { rng.gen_range(1, 8) as i32 } else { 0 };
        self.data.glass_color = rng.gen_range(0, 5) as i32;
        self.data.beard_mustache = if rng.gen_range(0, 3) == 0 { rng.gen_range(1, 3) as i32 } else { 0 };
        self.data.beard_type = if rng.gen_range(0, 3) == 0 { rng.gen_range(1, 3) as i32 } else { 0 };
        self.data.beard_color = self.data.hair_color;
    }
}

/// 快速轻量伪随机发生器 (Xorshift64)
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0x853C49E6748FEA9B } else { seed } }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn gen_range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max { return min; }
        min + (self.next_u64() % (max - min + 1))
    }
}
