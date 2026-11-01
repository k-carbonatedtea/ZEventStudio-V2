//! UMii 纯数据结构定义与 AAMP (YAML / 二进制) 序列化/反序列化核心

use roead::aamp::ParameterIO;
use serde::{Deserialize, Serialize};

/// UMii 角色配置数据核心结构 (与 BotW/TotK .bumii AAMP 格式字段严格对齐)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UmiiData {
    pub name: String,
    pub creator: String,

    // ffsd
    pub ffsd_no_use_ffsd: bool,
    pub ffsd_type: i32,

    // body
    pub body_race: i32,
    pub body_type: i32,
    pub body_number: i32,
    pub body_weight: i32,
    pub body_height: i32,

    // personal
    pub personal_sex_age: i32,
    pub personal_fav_color: i32,
    pub personal_sub_color_1: i32,
    pub personal_sub_color_2: i32,
    pub personal_head_fav_color: i32,
    pub personal_shoulder_fav_color: i32,
    pub personal_shoulder_sub_color_1: i32,
    pub personal_personality: String,
    pub personal_voice_type: String,

    // common
    pub common_backpack: i32,
    pub common_hat: i32,
    pub common_no_hat_always: bool,
    pub common_body_correct: i32,
    pub common_is_mid_age: bool,
    pub common_rot_cravicle: f32,
    pub common_rot_arm: f32,
    pub common_rot_leg: f32,
    pub common_rot_crotch: f32,

    // shape
    pub shape_jaw: i32,
    pub shape_wrinkle: i32,
    pub shape_make: i32,
    pub shape_trans_v: f32,
    pub shape_scale: f32,
    pub shape_skin_color: i32,

    // hair
    pub hair_type: i32,
    pub hair_color: i32,
    pub hair_flip: bool,

    // eye
    pub eye_type: i32,
    pub eye_color: i32,
    pub eye_trans_v: f32,
    pub eye_trans_u: f32,
    pub eye_rotate: f32,
    pub eye_scale: f32,
    pub eye_aspect: f32,
    pub eye_eyeball_trans_u: f32,
    pub eye_eyeball_trans_v: f32,
    pub eye_eyeball_scale: f32,
    pub eye_highlight_bright: i32,

    // eye_ctrl
    pub eye_ctrl_base_offset: [f32; 3],
    pub eye_ctrl_translim_out: f32,
    pub eye_ctrl_translim_in: f32,
    pub eye_ctrl_translim_d: f32,
    pub eye_ctrl_translim_u: f32,
    pub eye_ctrl_neck_offset_ud: f32,

    // eyebrow
    pub eyebrow_type: i32,
    pub eyebrow_color: i32,
    pub eyebrow_trans_v: f32,
    pub eyebrow_trans_u: f32,
    pub eyebrow_rotate: f32,
    pub eyebrow_scale: f32,
    pub eyebrow_aspect: f32,

    // nose
    pub nose_type: i32,
    pub nose_trans_v: f32,
    pub nose_scale: f32,

    // mouth
    pub mouth_type: i32,
    pub mouth_color: i32,
    pub mouth_trans_v: f32,
    pub mouth_scale: f32,
    pub mouth_aspect: f32,

    // beard
    pub beard_mustache: i32,
    pub beard_scale: f32,
    pub beard_type: i32,
    pub beard_color: i32,

    // glass
    pub glass_type: i32,
    pub glass_color: i32,

    // korog
    pub korog_mask: i32,
    pub korog_skin_color: i32,
    pub korog_left_plant: i32,
    pub korog_right_plant: i32,

    // goron
    pub goron_skin_color: i32,

    // gerudo
    pub gerudo_hair: i32,
    pub gerudo_hair_color: i32,
    pub gerudo_glass: i32,
    pub gerudo_glass_color: i32,
    pub gerudo_skin_color: i32,
    pub gerudo_lip_color: i32,

    // rito
    pub rito_body_color: i32,
    pub rito_hair_color: i32,

    // zora
    pub zora_body_color: i32,
}

impl Default for UmiiData {
    fn default() -> Self {
        Self {
            name: "DefaultNPC".to_string(),
            creator: "ZEventStudio".to_string(),
            ffsd_no_use_ffsd: false,
            ffsd_type: 0,
            body_race: 0,
            body_type: 0,
            body_number: 0,
            body_weight: 1,
            body_height: 0,
            personal_sex_age: 1,
            personal_fav_color: 0,
            personal_sub_color_1: -1,
            personal_sub_color_2: -1,
            personal_head_fav_color: -1,
            personal_shoulder_fav_color: -1,
            personal_shoulder_sub_color_1: -1,
            personal_personality: "Man_Normal".to_string(),
            personal_voice_type: "Hylia_Man_Young_Normal03".to_string(),
            common_backpack: -1,
            common_hat: -1,
            common_no_hat_always: false,
            common_body_correct: 0,
            common_is_mid_age: false,
            common_rot_cravicle: 0.0,
            common_rot_arm: 0.0,
            common_rot_leg: 0.0,
            common_rot_crotch: 0.0,
            shape_jaw: 0,
            shape_wrinkle: 0,
            shape_make: 0,
            shape_trans_v: 0.0,
            shape_scale: 1.0,
            shape_skin_color: 0,
            hair_type: 0,
            hair_color: 0,
            hair_flip: false,
            eye_type: 0,
            eye_color: 0,
            eye_trans_v: 12.0,
            eye_trans_u: 2.0,
            eye_rotate: 0.0,
            eye_scale: 4.0,
            eye_aspect: 3.0,
            eye_eyeball_trans_u: 0.0,
            eye_eyeball_trans_v: 0.0,
            eye_eyeball_scale: 0.03,
            eye_highlight_bright: 1,
            eye_ctrl_base_offset: [0.0, 0.02, 0.0],
            eye_ctrl_translim_out: 0.2,
            eye_ctrl_translim_in: 0.12,
            eye_ctrl_translim_d: 0.3,
            eye_ctrl_translim_u: 0.1,
            eye_ctrl_neck_offset_ud: 0.0,
            eyebrow_type: 0,
            eyebrow_color: 0,
            eyebrow_trans_v: 3.0,
            eyebrow_trans_u: 3.0,
            eyebrow_rotate: 0.0,
            eyebrow_scale: 3.0,
            eyebrow_aspect: 3.0,
            nose_type: 0,
            nose_trans_v: 9.0,
            nose_scale: 4.0,
            mouth_type: 0,
            mouth_color: 0,
            mouth_trans_v: 15.0,
            mouth_scale: 4.0,
            mouth_aspect: 3.0,
            beard_mustache: 0,
            beard_scale: 4.0,
            beard_type: 0,
            beard_color: 0,
            glass_type: 0,
            glass_color: 0,
            korog_mask: 0,
            korog_skin_color: 0,
            korog_left_plant: 0,
            korog_right_plant: 0,
            goron_skin_color: 0,
            gerudo_hair: 0,
            gerudo_hair_color: 0,
            gerudo_glass: 0,
            gerudo_glass_color: 0,
            gerudo_skin_color: 0,
            gerudo_lip_color: 0,
            rito_body_color: 0,
            rito_hair_color: -1,
            zora_body_color: 0,
        }
    }
}

impl UmiiData {
    /// 序列化为 BotW 标准 AAMP YAML 文本表达
    pub fn to_yaml_text(&self) -> String {
        format!(
            "!io\nversion: 0\ntype: xml\nparam_root: !list\n  objects:\n    ffsd: !obj\n      no_use_ffsd: {}\n      type: {}\n    body: !obj\n      race: {}\n      type: {}\n      number: {}\n      weight: {}\n      height: {}\n    personal: !obj\n      sex_age: {}\n      fav_color: {}\n      sub_color_1: {}\n      sub_color_2: {}\n      head_fav_color: {}\n      shoulder_fav_color: {}\n      shoulder_sub_color_1: {}\n      personality: {}\n      voice_type: {}\n    common: !obj\n      backpack: {}\n      hat: {}\n      no_hat_always: {}\n      body_correct: {}\n      is_mid_age: {}\n      rot_cravicle: {:.1}\n      rot_arm: {:.1}\n      rot_leg: {:.1}\n      rot_crotch: {:.1}\n    shape: !obj\n      jaw: {}\n      wrinkle: {}\n      make: {}\n      trans_v: {:.1}\n      scale: {:.1}\n      skin_color: {}\n    hair: !obj\n      type: {}\n      color: {}\n      flip: {}\n    eye: !obj\n      type: {}\n      color: {}\n      trans_v: {:.1}\n      trans_u: {:.1}\n      rotate: {:.1}\n      scale: {:.1}\n      aspect: {:.1}\n      eyeball_trans_u: {:.2}\n      eyeball_trans_v: {:.2}\n      eyeball_scale: {:.3}\n      highlight_bright: {}\n    eye_ctrl: !obj\n      base_offset: !vec3 [{:.2}, {:.2}, {:.2}]\n      translim_out: {:.2}\n      translim_in: {:.2}\n      translim_d: {:.2}\n      translim_u: {:.2}\n      neck_offset_ud: {:.2}\n    eyebrow: !obj\n      type: {}\n      color: {}\n      trans_v: {:.1}\n      trans_u: {:.1}\n      rotate: {:.1}\n      scale: {:.1}\n      aspect: {:.1}\n    nose: !obj\n      type: {}\n      trans_v: {:.1}\n      scale: {:.1}\n    mouth: !obj\n      type: {}\n      color: {}\n      trans_v: {:.1}\n      scale: {:.1}\n      aspect: {:.1}\n    beard: !obj\n      mustache: {}\n      scale: {:.1}\n      type: {}\n      color: {}\n    glass: !obj\n      type: {}\n      color: {}\n    korog: !obj\n      mask: {}\n      skin_color: {}\n      left_plant: {}\n      right_plant: {}\n    goron: !obj\n      skin_color: {}\n    gerudo: !obj\n      hair: {}\n      hair_color: {}\n      glass: {}\n      glass_color: {}\n      skin_color: {}\n      lip_color: {}\n    rito: !obj\n      body_color: {}\n      hair_color: {}\n    zora: !obj\n      body_color: {}\n  lists: {{}}\n",
            self.ffsd_no_use_ffsd, self.ffsd_type,
            self.body_race, self.body_type, self.body_number, self.body_weight, self.body_height,
            self.personal_sex_age, self.personal_fav_color, self.personal_sub_color_1, self.personal_sub_color_2,
            self.personal_head_fav_color, self.personal_shoulder_fav_color, self.personal_shoulder_sub_color_1,
            self.personal_personality, self.personal_voice_type,
            self.common_backpack, self.common_hat, self.common_no_hat_always, self.common_body_correct, self.common_is_mid_age,
            self.common_rot_cravicle, self.common_rot_arm, self.common_rot_leg, self.common_rot_crotch,
            self.shape_jaw, self.shape_wrinkle, self.shape_make, self.shape_trans_v, self.shape_scale, self.shape_skin_color,
            self.hair_type, self.hair_color, self.hair_flip,
            self.eye_type, self.eye_color, self.eye_trans_v, self.eye_trans_u, self.eye_rotate, self.eye_scale, self.eye_aspect,
            self.eye_eyeball_trans_u, self.eye_eyeball_trans_v, self.eye_eyeball_scale, self.eye_highlight_bright,
            self.eye_ctrl_base_offset[0], self.eye_ctrl_base_offset[1], self.eye_ctrl_base_offset[2],
            self.eye_ctrl_translim_out, self.eye_ctrl_translim_in, self.eye_ctrl_translim_d, self.eye_ctrl_translim_u, self.eye_ctrl_neck_offset_ud,
            self.eyebrow_type, self.eyebrow_color, self.eyebrow_trans_v, self.eyebrow_trans_u, self.eyebrow_rotate, self.eyebrow_scale, self.eyebrow_aspect,
            self.nose_type, self.nose_trans_v, self.nose_scale,
            self.mouth_type, self.mouth_color, self.mouth_trans_v, self.mouth_scale, self.mouth_aspect,
            self.beard_mustache, self.beard_scale, self.beard_type, self.beard_color,
            self.glass_type, self.glass_color,
            self.korog_mask, self.korog_skin_color, self.korog_left_plant, self.korog_right_plant,
            self.goron_skin_color,
            self.gerudo_hair, self.gerudo_hair_color, self.gerudo_glass, self.gerudo_glass_color, self.gerudo_skin_color, self.gerudo_lip_color,
            self.rito_body_color, self.rito_hair_color,
            self.zora_body_color,
        )
    }

    /// 编译为标准任天堂 AAMP 二进制字节流 (.bumii)
    pub fn to_binary(&self) -> Result<Vec<u8>, String> {
        let text = self.to_yaml_text();
        let pio = ParameterIO::from_text(&text)
            .map_err(|e| format!("序列化 AAMP 失败: {:?}", e))?;
        Ok(pio.to_binary().to_vec())
    }

    /// 从任天堂 AAMP 二进制数据反序列化为 UmiiData
    pub fn from_binary(data: &[u8]) -> Result<Self, String> {
        let pio = ParameterIO::from_binary(data)
            .map_err(|e| format!("解析 AAMP 二进制失败: {:?}", e))?;
        Self::from_yaml_text(&pio.to_text())
    }

    /// 从 YAML 文本中解析参数字段
    pub fn from_yaml_text(text: &str) -> Result<Self, String> {
        let mut data = Self::default();
        let mut current_obj = String::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("!io") || trimmed.starts_with("param_root") {
                continue;
            }

            if let Some(obj_name) = trimmed.strip_suffix(": !obj") {
                current_obj = obj_name.trim().to_string();
                continue;
            }

            let Some((key, val_str)) = trimmed.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let val = val_str.trim().trim_matches(|c| c == '\'' || c == '"');

            data.assign_field(&current_obj, key, val);
        }

        Ok(data)
    }

    /// 根据当前所属对象与字段名称赋值
    fn assign_field(&mut self, obj: &str, key: &str, val: &str) {
        match (obj, key) {
            ("body", "race") => self.body_race = val.parse().unwrap_or(0),
            ("body", "type") => self.body_type = val.parse().unwrap_or(0),
            ("body", "number") => self.body_number = val.parse().unwrap_or(0),
            ("body", "weight") => self.body_weight = val.parse().unwrap_or(1),
            ("body", "height") => self.body_height = val.parse().unwrap_or(0),
            ("personal", "sex_age") => self.personal_sex_age = val.parse().unwrap_or(1),
            ("personal", "fav_color") => self.personal_fav_color = val.parse().unwrap_or(0),
            ("personal", "personality") => self.personal_personality = val.to_string(),
            ("personal", "voice_type") => self.personal_voice_type = val.to_string(),
            ("common", "backpack") => self.common_backpack = val.parse().unwrap_or(-1),
            ("common", "hat") => self.common_hat = val.parse().unwrap_or(-1),
            ("common", "no_hat_always") => self.common_no_hat_always = val == "true",
            ("shape", "jaw") => self.shape_jaw = val.parse().unwrap_or(0),
            ("shape", "wrinkle") => self.shape_wrinkle = val.parse().unwrap_or(0),
            ("shape", "make") => self.shape_make = val.parse().unwrap_or(0),
            ("shape", "skin_color") => self.shape_skin_color = val.parse().unwrap_or(0),
            ("shape", "scale") => self.shape_scale = val.parse().unwrap_or(1.0),
            ("shape", "trans_v") => self.shape_trans_v = val.parse().unwrap_or(0.0),
            ("hair", "type") => self.hair_type = val.parse().unwrap_or(0),
            ("hair", "color") => self.hair_color = val.parse().unwrap_or(0),
            ("hair", "flip") => self.hair_flip = val == "true",
            ("eye", "type") => self.eye_type = val.parse().unwrap_or(0),
            ("eye", "color") => self.eye_color = val.parse().unwrap_or(0),
            ("eye", "rotate") => self.eye_rotate = val.parse().unwrap_or(0.0),
            ("eye", "scale") => self.eye_scale = val.parse().unwrap_or(4.0),
            ("eye", "trans_u") => self.eye_trans_u = val.parse().unwrap_or(2.0),
            ("eye", "trans_v") => self.eye_trans_v = val.parse().unwrap_or(12.0),
            ("eyebrow", "type") => self.eyebrow_type = val.parse().unwrap_or(0),
            ("eyebrow", "color") => self.eyebrow_color = val.parse().unwrap_or(0),
            ("eyebrow", "rotate") => self.eyebrow_rotate = val.parse().unwrap_or(0.0),
            ("eyebrow", "scale") => self.eyebrow_scale = val.parse().unwrap_or(3.0),
            ("nose", "type") => self.nose_type = val.parse().unwrap_or(0),
            ("nose", "scale") => self.nose_scale = val.parse().unwrap_or(4.0),
            ("nose", "trans_v") => self.nose_trans_v = val.parse().unwrap_or(9.0),
            ("mouth", "type") => self.mouth_type = val.parse().unwrap_or(0),
            ("mouth", "color") => self.mouth_color = val.parse().unwrap_or(0),
            ("mouth", "scale") => self.mouth_scale = val.parse().unwrap_or(4.0),
            ("beard", "mustache") => self.beard_mustache = val.parse().unwrap_or(0),
            ("beard", "type") => self.beard_type = val.parse().unwrap_or(0),
            ("beard", "color") => self.beard_color = val.parse().unwrap_or(0),
            ("beard", "scale") => self.beard_scale = val.parse().unwrap_or(4.0),
            ("glass", "type") => self.glass_type = val.parse().unwrap_or(0),
            ("glass", "color") => self.glass_color = val.parse().unwrap_or(0),
            ("korog", "mask") => self.korog_mask = val.parse().unwrap_or(0),
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_umii_data() {
        let d = UmiiData::default();
        assert_eq!(d.name, "DefaultNPC");
        assert_eq!(d.creator, "ZEventStudio");
        assert_eq!(d.body_race, 0);
    }

    #[test]
    fn test_yaml_roundtrip() {
        let mut d = UmiiData::default();
        d.hair_type = 42;
        d.shape_jaw = 5;
        d.glass_type = 3;

        let yaml = d.to_yaml_text();
        assert!(yaml.contains("type: 42"));
        assert!(yaml.contains("jaw: 5"));

        let parsed = UmiiData::from_yaml_text(&yaml).expect("Failed to parse YAML");
        assert_eq!(parsed.hair_type, 42);
        assert_eq!(parsed.shape_jaw, 5);
        assert_eq!(parsed.glass_type, 3);
    }

    #[test]
    fn test_binary_roundtrip() {
        let mut d = UmiiData::default();
        d.hair_type = 15;
        let bin = d.to_binary().expect("Failed to serialize to AAMP");
        assert!(bin.starts_with(b"AAMP"));

        let loaded = UmiiData::from_binary(&bin).expect("Failed to load binary");
        assert_eq!(loaded.hair_type, 15);
    }
}
