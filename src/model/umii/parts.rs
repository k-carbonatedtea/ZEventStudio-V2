//! UMii 3D 模型部件目标解析、发型回退映射与模型目录探测

use super::data::UmiiData;
use super::synthesizer::ModelPlatform;
use crate::model::settings::GamePathSettings;
use std::path::{Path, PathBuf};

/// 任天堂官方发型回退映射字典 (来自 Bootup.pack/Mii/UMiiConstructionInfo.byml)
pub const HAIR_FALLBACK_TABLE: &[(i32, i32)] = &[
    (1, 29),
    (2, 18),
    (3, 27),
    (5, 98),
    (7, 15),
    (8, 27),
    (9, 97),
    (10, 12),
    (11, 98),
    (13, 98),
    (14, 27),
    (16, 12),
    (17, 98),
    (19, 104),
    (20, 12),
    (22, 27),
    (23, 36),
    (24, 6),
    (26, 109),
    (31, 75),
    (32, 40),
    (33, 40),
    (34, 45),
    (37, 40),
    (39, 36),
    (43, 38),
    (44, 52),
    (47, 40),
    (48, 55),
    (49, 38),
    (50, 52),
    (53, 15),
    (56, 75),
    (57, 45),
    (58, 116),
    (59, 36),
    (60, 98),
    (61, 77),
    (62, 51),
    (63, 98),
    (64, 75),
    (66, 52),
    (67, 36),
    (68, 75),
    (70, 55),
    (71, 15),
    (72, 123),
    (73, 75),
    (74, 123),
    (76, 75),
    (79, 127),
    (80, 89),
    (84, 130),
    (85, 15),
    (86, 88),
    (87, 51),
    (90, 0),
    (91, 29),
    (92, 18),
    (93, 4),
    (94, 98),
    (95, 12),
    (96, 15),
    (99, 98),
    (100, 27),
    (101, 15),
    (102, 98),
    (103, 18),
    (105, 12),
    (106, 21),
    (107, 40),
    (108, 25),
    (110, 27),
    (111, 28),
    (112, 29),
    (113, 35),
    (114, 46),
    (115, 75),
    (118, 54),
    (119, 51),
    (120, 98),
    (121, 75),
    (122, 69),
    (124, 123),
    (125, 75),
    (126, 78),
    (128, 82),
    (129, 82),
    (131, 15),
];

/// 单个待合成的 3D 部件元信息
#[derive(Debug, Clone)]
pub struct UmiiPartTarget {
    pub category: &'static str,
    pub primary_filename: String,
    pub fallback_filenames: Vec<String>,
}

/// 获取发型在游戏资产中的真实索引 (若未直接建模则经由映射表回退)
pub fn resolve_hair_index(hair_type: i32) -> i32 {
    for &(src, dst) in HAIR_FALLBACK_TABLE {
        if src == hair_type {
            return dst;
        }
    }
    hair_type
}

/// 收集所有候选搜索模型目录 (支持工程根目录、用户全局配置与本地测试路径)
pub fn collect_model_search_dirs(
    platform: ModelPlatform,
    project_root: Option<&str>,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // 1. 当前工程根目录
    if let Some(root_str) = project_root {
        let p = Path::new(root_str);
        dirs.push(p.join("Model"));
        dirs.push(p.join("Romfs").join("Model"));
        dirs.push(p.join("romfs").join("Model"));
        dirs.push(p.join("content").join("Model"));
        dirs.push(p.to_path_buf());
    }

    // 2. 用户全局路径设置
    let settings = GamePathSettings::load_from_disk();
    match platform {
        ModelPlatform::Switch => {
            if !settings.switch.game_dir.is_empty() {
                dirs.push(Path::new(&settings.switch.game_dir).join("Model"));
                dirs.push(
                    Path::new(&settings.switch.game_dir)
                        .join("romfs")
                        .join("Model"),
                );
            }
            dirs.push(PathBuf::from(
                r"C:\Users\Administrator\Desktop\1.8.9Xe\01007EF00011E000\romfs\Model",
            ));
        }
        ModelPlatform::WiiU => {
            if !settings.wiiu.game_dir.is_empty() {
                dirs.push(Path::new(&settings.wiiu.game_dir).join("Model"));
                dirs.push(
                    Path::new(&settings.wiiu.game_dir)
                        .join("content")
                        .join("Model"),
                );
            }
            if !settings.wiiu.update_dir.is_empty() {
                dirs.push(Path::new(&settings.wiiu.update_dir).join("Model"));
            }
            dirs.push(PathBuf::from(
                r"C:\Users\Administrator\Desktop\Tes\Romfs\Model",
            ));
            dirs.push(PathBuf::from(r"C:\Users\Administrator\Desktop\Tes\Model"));
        }
    }

    // 兜底双向互通搜索
    dirs.push(PathBuf::from(
        r"C:\Users\Administrator\Desktop\Tes\Romfs\Model",
    ));
    dirs.push(PathBuf::from(
        r"C:\Users\Administrator\Desktop\1.8.9Xe\01007EF00011E000\romfs\Model",
    ));

    dirs.retain(|d| d.exists());
    dirs
}

/// 根据 UMii 参数生成各部件的目标文件名与回退候选项
pub fn generate_part_targets(data: &UmiiData) -> Vec<UmiiPartTarget> {
    let mut targets = Vec::new();
    let race_str = match data.body_race {
        1 => "Korogu",
        2 => "Goron",
        4 => "Gerudo",
        5 => "Zora",
        6 => "Rito",
        _ => "Hylia",
    };

    let gender_code = match data.personal_sex_age {
        0 => "B", // Boy Child
        2 => "X", // Old Man (fallback M)
        3 => "G", // Girl Child
        4 => "W", // Woman Adult
        5 => "Y", // Old Woman (fallback W)
        _ => "M", // Man Adult
    };

    let body_type_code = match data.body_type {
        1 => "S",  // Stablehand
        2 => "T",  // Traveler
        3 => "N",  // Guard / Soldier
        4 => "SK", // Sheikah
        _ => "C",  // Citizen / Normal
    };

    // 1. 身体 (Body)
    let body_primary = format!(
        "UMii_{}_Body{}_{}_{:03}.sbfres",
        race_str, body_type_code, gender_code, data.body_number
    );
    let mut body_fallbacks = vec![
        format!("UMii_{}_BodyC_{}_000.sbfres", race_str, gender_code),
        format!("UMii_{}_Body.sbfres", race_str),
    ];
    if gender_code == "X" {
        body_fallbacks.insert(
            0,
            format!(
                "UMii_{}_Body{}_M_{:03}.sbfres",
                race_str, body_type_code, data.body_number
            ),
        );
    } else if gender_code == "Y" {
        body_fallbacks.insert(
            0,
            format!(
                "UMii_{}_Body{}_W_{:03}.sbfres",
                race_str, body_type_code, data.body_number
            ),
        );
    }
    targets.push(UmiiPartTarget {
        category: "Body",
        primary_filename: body_primary,
        fallback_filenames: body_fallbacks,
    });

    // 2. 脸型 (Face)
    let jaw_mult = data.shape_jaw.clamp(0, 11) * 3;
    let face_primary = format!(
        "UMii_{}_Face_{}_{:03}.sbfres",
        race_str, gender_code, jaw_mult
    );
    let mut face_fallbacks = vec![
        format!("UMii_{}_Face_{}_000.sbfres", race_str, gender_code),
        format!("UMii_{}_Face_000.sbfres", race_str),
    ];
    if gender_code == "X" {
        face_fallbacks.insert(
            0,
            format!("UMii_{}_Face_M_{:03}.sbfres", race_str, jaw_mult),
        );
    } else if gender_code == "Y" {
        face_fallbacks.insert(
            0,
            format!("UMii_{}_Face_W_{:03}.sbfres", race_str, jaw_mult),
        );
    }
    targets.push(UmiiPartTarget {
        category: "Face",
        primary_filename: face_primary,
        fallback_filenames: face_fallbacks,
    });

    // 3. 发型 (Hair)
    let resolved_hair = resolve_hair_index(data.hair_type);
    let hair_primary = format!("UMii_{}_Hair_C_{:03}.sbfres", race_str, resolved_hair);
    let hair_fallbacks = vec![
        format!("UMii_{}_Hair_C_000.sbfres", race_str),
        format!("UMii_{}_Hair_000.sbfres", race_str),
    ];
    targets.push(UmiiPartTarget {
        category: "Hair",
        primary_filename: hair_primary,
        fallback_filenames: hair_fallbacks,
    });

    // 4. 鼻型 (Nose)
    let nose_primary = format!(
        "UMii_{}_Nose_C_{:03}.sbfres",
        race_str,
        data.nose_type.clamp(0, 17)
    );
    targets.push(UmiiPartTarget {
        category: "Nose",
        primary_filename: nose_primary,
        fallback_filenames: vec![format!("UMii_{}_Nose_C_000.sbfres", race_str)],
    });

    // 5. 胡须 (Beard / Mustache)
    if data.beard_mustache > 0 {
        targets.push(UmiiPartTarget {
            category: "Mustache",
            primary_filename: format!(
                "UMii_{}_Mustache_C_{:03}.sbfres",
                race_str, data.beard_mustache
            ),
            fallback_filenames: vec![],
        });
    }
    if data.beard_type > 0 {
        targets.push(UmiiPartTarget {
            category: "Beard",
            primary_filename: format!("UMii_{}_Beard_C_{:03}.sbfres", race_str, data.beard_type),
            fallback_filenames: vec![],
        });
    }

    // 6. 眼镜 (Glasses)
    if data.glass_type > 0 {
        targets.push(UmiiPartTarget {
            category: "Glasses",
            primary_filename: format!("UMii_{}_Glass_C_{:03}.sbfres", race_str, data.glass_type),
            fallback_filenames: vec![],
        });
    }

    // 7. 特殊种族装饰 (如克洛格面具与植物)
    if data.body_race == 1 {
        if data.korog_mask >= 0 {
            targets.push(UmiiPartTarget {
                category: "KorogMask",
                primary_filename: format!("UMii_Korogu_Mask_{:03}.sbfres", data.korog_mask),
                fallback_filenames: vec!["UMii_Korogu_Mask_000.sbfres".to_string()],
            });
        }
        if data.korog_left_plant >= 0 {
            targets.push(UmiiPartTarget {
                category: "KorogPlant",
                primary_filename: format!("UMii_Korogu_Plant_{:03}.sbfres", data.korog_left_plant),
                fallback_filenames: vec![],
            });
        }
    }

    targets
}

/// 在候选目录列表中检索目标部件对应的物理文件路径
pub fn locate_part_file(target: &UmiiPartTarget, search_dirs: &[PathBuf]) -> Option<PathBuf> {
    let mut names = vec![target.primary_filename.as_str()];
    for fb in &target.fallback_filenames {
        names.push(fb.as_str());
    }

    for dir in search_dirs {
        for name in &names {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
            let p_bfres = dir.join(name.replace(".sbfres", ".bfres"));
            if p_bfres.is_file() {
                return Some(p_bfres);
            }
        }
    }
    None
}
