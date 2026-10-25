use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use gpui::SharedString;
use roead::byml::Byml;
use roead::sarc::Sarc;
use roead::yaz0;

use crate::model::settings::GamePathSettings;
use crate::ui::views::quest_map::markers_data::{
    get_default_hyrule_markers, HyruleMarker, HyruleMarkerKind,
};
use crate::ui::views::quest_map::markers_msbt::{
    extract_location_msbt_from_pack, parse_location_msbt,
};
use crate::ui::views::quest_map::markers_shops::resolve_shop_marker;

/// 从已配置的游戏路径与当前项目目录中自动提取海拉鲁地图标点（神庙、塔、驿站、城镇及自定义 Mod 标点）
pub fn extract_hyrule_markers(
    project_path: Option<&str>,
    game_paths: Option<&GamePathSettings>,
) -> Vec<HyruleMarker> {
    extract_hyrule_markers_with_progress(project_path, game_paths, |_| {})
}

/// 支持实时汇报当前扫描与解压文件进度的海拉鲁标点提取器
pub fn extract_hyrule_markers_with_progress<F>(
    project_path: Option<&str>,
    game_paths: Option<&GamePathSettings>,
    mut on_progress: F,
) -> Vec<HyruleMarker>
where
    F: FnMut(&str),
{
    on_progress("正在加载基准地标字典...");
    // 1. 初始化基准表（内置 136 座官方神庙与地标）
    let mut markers_map: HashMap<String, HyruleMarker> = HashMap::new();
    for marker in get_default_hyrule_markers() {
        markers_map.insert(marker.id.to_string(), marker);
    }

    let mut dict_zh: HashMap<String, String> = HashMap::new();
    let mut dict_en: HashMap<String, String> = HashMap::new();

    // 2. 从已配置的游戏 Dump 路径中提取（严格优先使用当前所选平台）
    if let Some(gp) = game_paths {
        let mut primary_dirs = Vec::new();
        let mut fallback_dirs = Vec::new();

        match gp.current_platform {
            crate::model::settings::PlatformType::Switch => {
                if !gp.switch.game_dir.trim().is_empty() {
                    primary_dirs.push(PathBuf::from(gp.switch.game_dir.trim()));
                }
                if !gp.switch.dlc_dir.trim().is_empty() {
                    let dlc = gp.switch.dlc_dir.trim();
                    primary_dirs.push(PathBuf::from(dlc));
                    if dlc.contains("01007EF00011F001") {
                        primary_dirs.push(PathBuf::from(dlc.replace("01007EF00011F001", "01007EF00011E000")));
                    }
                }
                primary_dirs.push(PathBuf::from(r"C:\Game\BOTW\01007EF00011E000\romfs"));
                if !gp.wiiu.update_dir.trim().is_empty() {
                    fallback_dirs.push(PathBuf::from(gp.wiiu.update_dir.trim()));
                }
                if !gp.wiiu.game_dir.trim().is_empty() {
                    fallback_dirs.push(PathBuf::from(gp.wiiu.game_dir.trim()));
                }
            }
            crate::model::settings::PlatformType::WiiU => {
                if !gp.wiiu.update_dir.trim().is_empty() {
                    primary_dirs.push(PathBuf::from(gp.wiiu.update_dir.trim()));
                }
                if !gp.wiiu.game_dir.trim().is_empty() {
                    primary_dirs.push(PathBuf::from(gp.wiiu.game_dir.trim()));
                }
                if !gp.switch.game_dir.trim().is_empty() {
                    fallback_dirs.push(PathBuf::from(gp.switch.game_dir.trim()));
                }
            }
        }

        // 先扫描当前平台有效目录
        let mut scanned_any = false;
        for dir in primary_dirs {
            if dir.exists() {
                scanned_any = true;
                for marker in scan_static_markers_in_dir(&dir, false, 3, &mut dict_zh, &mut dict_en, &mut on_progress) {
                    merge_marker_into_map(&mut markers_map, marker);
                }
            }
        }

        // 仅当当前平台未能扫描到任何有效目录时，才使用备选平台
        if !scanned_any {
            for dir in fallback_dirs {
                if dir.exists() {
                    for marker in scan_static_markers_in_dir(&dir, false, 3, &mut dict_zh, &mut dict_en, &mut on_progress) {
                        merge_marker_into_map(&mut markers_map, marker);
                    }
                }
            }
        }
    }

    // 3. 从当前打开的项目中提取（最高优先级：用户自定义神庙、自定义地图 Mod）
    if let Some(proj) = project_path {
        let proj_buf = PathBuf::from(proj);
        if proj_buf.exists() {
            let proj_name = proj_buf.file_name().and_then(|n| n.to_str()).unwrap_or("Project");
            on_progress(&format!("扫描项目自定义标点: {}", proj_name));
            for marker in scan_static_markers_in_dir(&proj_buf, true, 4, &mut dict_zh, &mut dict_en, &mut on_progress) {
                merge_marker_into_map(&mut markers_map, marker);
            }
        }
    }

    // 4. 多语言地名全量注入与润色 (使用官方 LocationMarker.msbt 注入官方中文名与英文名)
    for marker in markers_map.values_mut() {
        if marker.kind == HyruleMarkerKind::Shop || marker.is_custom {
            continue;
        }
        let id_str = marker.id.as_ref();
        if let Some(zh) = dict_zh.get(id_str) {
            marker.name_zh = SharedString::from(zh.clone());
        }
        if let Some(en) = dict_en.get(id_str) {
            marker.name_en = SharedString::from(en.clone());
        }
    }

    on_progress(&format!("标点同步完成 (共 {} 个有效标点)", markers_map.len()));

    // 转为向量并保持稳定顺序
    let mut result: Vec<HyruleMarker> = markers_map.into_values().collect();
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}

/// 将提取到的标点合并到映射表中 (智能辨识原版地标与真正新增的 Mod 地标)
fn merge_marker_into_map(map: &mut HashMap<String, HyruleMarker>, new_marker: HyruleMarker) {
    let key = new_marker.id.to_string();
    if let Some(existing) = map.get_mut(&key) {
        // 该标点在官方基准地标中已经存在 (说明是已知官方 136 座地标之一)
        let dx = existing.position.0 - new_marker.position.0;
        let dy = existing.position.1 - new_marker.position.1;
        let dist = (dx * dx + dy * dy).sqrt();

        if new_marker.is_custom {
            // 来自已打开项目扫描：
            // 只有当标点世界坐标被 Mod 作者实质性挪移 (偏离原版基准 > 50 米) 时，才视为 Mod 搬迁的地标
            if dist > 50.0 {
                existing.position = new_marker.position;
                existing.altitude = new_marker.altitude;
                existing.is_custom = true;
            } else {
                // 坐标基本一致 (<= 50 米)，说明只是项目 romfs 中携带的官方原版副本
                // 坚决保持原版优雅中文名与试炼名，绝不标记为 is_custom，避免污染和全景卡片泛滥
            }
        } else {
            // 来自官方 Dump 的同步更新
            existing.position = new_marker.position;
            existing.altitude = new_marker.altitude;
        }
    } else {
        // 全新自定义标点 (基准表中不存在的新 ID，如自创神庙 Dungeon137、自定义营地等)
        map.insert(key, new_marker);
    }
}

/// 扫描指定目录下的 Static.smubin、Bootup.pack 及语言包
fn scan_static_markers_in_dir<F>(
    root: &Path,
    is_custom: bool,
    max_depth: usize,
    dict_zh: &mut HashMap<String, String>,
    dict_en: &mut HashMap<String, String>,
    on_progress: &mut F,
) -> Vec<HyruleMarker>
where
    F: FnMut(&str),
{
    let mut markers = Vec::new();
    let mut dirs_to_visit = vec![(root.to_path_buf(), 0)];

    while let Some((curr, depth)) = dirs_to_visit.pop() {
        if depth > max_depth {
            continue;
        }

        let Ok(entries) = fs::read_dir(&curr) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();

            if path.is_dir() {
                // 跳过无关超大编译目录
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    dirs_to_visit.push((path, depth + 1));
                }
            } else if path.is_file() {
                if name.eq_ignore_ascii_case("Static.smubin") {
                    let display_path = path.strip_prefix(root).unwrap_or(&path).display().to_string();
                    on_progress(&format!("读取文件 {}", display_path));
                    if let Ok(bytes) = fs::read(&path) {
                        on_progress(&format!("解析地标数据 {}", display_path));
                        markers.extend(parse_location_markers_from_smubin(&bytes, is_custom));
                    }
                } else if name.eq_ignore_ascii_case("Bootup.pack") {
                    let display_path = path.strip_prefix(root).unwrap_or(&path).display().to_string();
                    on_progress(&format!("读取文件 {}", display_path));
                    if let Ok(bytes) = fs::read(&path) {
                        on_progress(&format!("解压 Yaz0: {}", display_path));
                        markers.extend(parse_markers_from_bootup_pack(&bytes, is_custom, on_progress));
                    }
                } else if name.eq_ignore_ascii_case("Bootup_CNzh.pack") || name.eq_ignore_ascii_case("Bootup_TWzh.pack") {
                    if let Ok(bytes) = fs::read(&path) {
                        on_progress(&format!("提取地名语言包: {}", name));
                        dict_zh.extend(extract_location_msbt_from_pack(&bytes));
                    }
                } else if name.eq_ignore_ascii_case("Bootup_USen.pack") || name.eq_ignore_ascii_case("Bootup_EUen.pack") {
                    if let Ok(bytes) = fs::read(&path) {
                        on_progress(&format!("提取英文地名包: {}", name));
                        dict_en.extend(extract_location_msbt_from_pack(&bytes));
                    }
                } else if name.eq_ignore_ascii_case("LocationMarker.msbt") {
                    if let Ok(bytes) = fs::read(&path) {
                        dict_zh.extend(parse_location_msbt(&bytes));
                    }
                }
            }
        }
    }

    markers
}

/// 从 Bootup.pack 封包中提取 Static.smubin 并解析
fn parse_markers_from_bootup_pack<F>(
    pack_bytes: &[u8],
    is_custom: bool,
    on_progress: &mut F,
) -> Vec<HyruleMarker>
where
    F: FnMut(&str),
{
    let decomp = yaz0::decompress(pack_bytes).unwrap_or_else(|_| pack_bytes.to_vec());
    on_progress("解析 SARC 封包结构...");
    let Ok(sarc) = Sarc::new(&decomp) else { return Vec::new() };

    for file in sarc.files() {
        if let Some(name) = file.name() {
            if name.ends_with("Map/MainField/Static.smubin") || name.eq_ignore_ascii_case("Static.smubin") {
                on_progress(&format!("提取并解析 {}", name));
                return parse_location_markers_from_smubin(file.data(), is_custom);
            }
        }
    }

    Vec::new()
}

/// 解析 Static.smubin 字节流中的 LocationMarker 列表
fn parse_location_markers_from_smubin(data: &[u8], is_custom: bool) -> Vec<HyruleMarker> {
    let mut markers = Vec::new();
    let decomp = yaz0::decompress(data).unwrap_or_else(|_| data.to_vec());
    let Ok(byml) = Byml::from_binary(&decomp) else { return markers };

    let Byml::Map(root_map) = byml else { return markers };
    let Some(Byml::Array(loc_markers)) = root_map.get("LocationMarker") else {
        return markers;
    };

    for item in loc_markers {
        let Byml::Map(entry) = item else { continue };

        // 1. 提取标识 ID
        let mid = entry.get("MessageID").and_then(|v| match v {
            Byml::String(s) => Some(s.as_str()),
            _ => None,
        }).unwrap_or_default();

        let save_flag = entry.get("SaveFlag").and_then(|v| match v {
            Byml::String(s) => Some(s.as_str()),
            _ => None,
        }).unwrap_or_default();

        let icon_str = entry.get("Icon").and_then(|v| match v {
            Byml::String(s) => Some(s.as_str()),
            _ => None,
        }).unwrap_or_default();

        // 提取世界坐标 (X, Z, Y)
        let (pos_x, pos_z, altitude) = if let Some(Byml::Map(t)) = entry.get("Translate") {
            let x = extract_float(t.get("X")).unwrap_or(0.0);
            let y = extract_float(t.get("Y")).unwrap_or(0.0);
            let z = extract_float(t.get("Z")).unwrap_or(0.0);
            (x, z, y)
        } else {
            (0.0, 0.0, 0.0)
        };

        // 1. 优先作为聚落商业店铺解析 (一始村及各大村落 37 家商铺独立命名与图标)
        if let Some(shop_marker) = resolve_shop_marker(save_flag, icon_str, (pos_x, pos_z), altitude, is_custom) {
            markers.push(shop_marker);
            continue;
        }

        let marker_id = if !mid.is_empty() {
            mid
        } else if !save_flag.is_empty() {
            save_flag.strip_prefix("Location_").unwrap_or(save_flag)
        } else {
            continue;
        };

        // 2. 提取常规地标类别与图标
        let (kind, icon_char) = match icon_str {
            "Dungeon" => (HyruleMarkerKind::Shrine, "⛩️"),
            "Tower" => (HyruleMarkerKind::Tower, "🗼"),
            "Hatago" => (HyruleMarkerKind::Stable, "🐎"),
            "Village" => (HyruleMarkerKind::Village, "🏡"),
            "Castle" => (HyruleMarkerKind::Facility, "🏰"),
            "Labo" => (HyruleMarkerKind::Facility, "🔬"),
            "StartPoint" => (HyruleMarkerKind::Facility, "✨"),
            s if s.starts_with("Remains") => (HyruleMarkerKind::DivineBeast, "🦅"),
            _ => (HyruleMarkerKind::Facility, "📍"),
        };

        let name_zh = if is_custom {
            format!("{} (Mod)", marker_id)
        } else {
            marker_id.to_string()
        };
        let name_en = marker_id.to_string();

        markers.push(HyruleMarker {
            id: SharedString::from(marker_id.to_string()),
            kind,
            name_zh: SharedString::from(name_zh),
            name_en: SharedString::from(name_en),
            sub_name_zh: is_custom.then(|| SharedString::from("自定义标点")),
            sub_name_en: is_custom.then(|| SharedString::from("Custom Marker")),
            position: (pos_x, pos_z),
            altitude,
            icon_char: SharedString::from(icon_char),
            is_custom,
        });
    }

    markers
}

/// 辅助提取 BYML 中的浮点数或整数并转为 f32
fn extract_float(val: Option<&Byml>) -> Option<f32> {
    match val {
        Some(Byml::Float(f)) => Some(*f),
        Some(Byml::Double(d)) => Some(*d as f32),
        Some(Byml::I32(i)) => Some(*i as f32),
        Some(Byml::U32(u)) => Some(*u as f32),
        _ => None,
    }
}
