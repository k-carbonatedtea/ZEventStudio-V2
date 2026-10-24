use std::path::PathBuf;
use crate::ui::views::quest_map::markers_data::{HyruleMarker, HyruleMarkerKind};

/// 解析海拉鲁地图官方游戏贴图文件的绝对或相对路径
fn resolve_map_asset(filename: &str) -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(r"C:\Users\Administrator\Desktop\Zevent\ZEventStudio\assets\map").join(filename),
        PathBuf::from("assets").join("map").join(filename),
    ];

    for cand in &candidates {
        if cand.exists() {
            return Some(cand.clone());
        }
    }

    if let Ok(mut exe) = std::env::current_exe() {
        exe.pop();
        let cand = exe.join("assets").join("map").join(filename);
        if cand.exists() {
            return Some(cand);
        }
    }

    None
}

/// 获取试炼神庙官方贴图路径 (支持常规态与激活高亮态)
pub fn get_shrine_icon_path(is_active: bool) -> Option<PathBuf> {
    if is_active {
        resolve_map_asset("shrine_active.png").or_else(|| resolve_map_asset("shrine_blue.png"))
    } else {
        resolve_map_asset("shrine_blue.png")
    }
}

/// 获取希卡之塔官方贴图路径
pub fn get_tower_icon_path() -> Option<PathBuf> {
    resolve_map_asset("tower_blue.png")
}

/// 获取海拉鲁马头驿站官方贴图路径
pub fn get_stable_icon_path() -> Option<PathBuf> {
    resolve_map_asset("stable.png")
}

/// 获取四大神兽官方发光贴图路径
pub fn get_beast_icon_path(marker_id: &str) -> Option<PathBuf> {
    let filename = match marker_id {
        "RemainsWind" => "beast_medoh.png",
        "RemainsElectric" => "beast_naboris.png",
        "RemainsFire" => "beast_rudania.png",
        "RemainsWater" => "beast_ruta.png",
        _ => return None,
    };
    resolve_map_asset(filename)
}

/// 获取城镇、要地、研究所及城堡官方贴图路径
pub fn get_settlement_icon_path(marker_id: &str, kind: HyruleMarkerKind) -> Option<PathBuf> {
    match marker_id {
        "HyruleCastle" => resolve_map_asset("castle.png"),
        "HatenoLabo" | "AncientLabo" => resolve_map_asset("labo.png"),
        "StartPoint" => resolve_map_asset("resurrection.png"),
        _ => {
            if kind == HyruleMarkerKind::Village {
                resolve_map_asset("village.png")
            } else {
                resolve_map_asset("castle.png")
            }
        }
    }
}

/// 统一获取任意海拉鲁地标的官方游戏图标贴图路径
pub fn get_marker_game_icon_path(marker: &HyruleMarker, is_active: bool) -> Option<PathBuf> {
    match marker.kind {
        HyruleMarkerKind::Shrine => get_shrine_icon_path(is_active),
        HyruleMarkerKind::Tower => get_tower_icon_path(),
        HyruleMarkerKind::Stable => get_stable_icon_path(),
        HyruleMarkerKind::DivineBeast => get_beast_icon_path(marker.id),
        HyruleMarkerKind::Village | HyruleMarkerKind::Facility => {
            get_settlement_icon_path(marker.id, marker.kind)
        }
    }
}
