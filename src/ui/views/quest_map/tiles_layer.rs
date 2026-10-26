use gpui::*;
use std::path::{Path, PathBuf};

/// 获取全局基准海拉鲁瓦片缓存目录 (优先使用应用内 .cache/map_tiles)
pub fn get_global_map_tiles_cache_dir() -> PathBuf {
    let local = PathBuf::from(r"C:\Users\Administrator\Desktop\Zevent\ZEventStudio\.cache\map_tiles");
    if local.exists() {
        return local;
    }
    if let Ok(mut exe) = std::env::current_exe() {
        exe.pop();
        let cand = exe.join(".cache").join("map_tiles");
        if cand.exists() {
            return cand;
        }
    }
    local
}

/// 获取指定已打开项目的专有瓦片缓存目录
pub fn get_project_tiles_cache_dir(project_path: &str) -> PathBuf {
    Path::new(project_path).join(".cache").join("map_tiles")
}

/// 检查给定目录是否为有效的 MapTex MainField 目录
fn is_valid_mainfield_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    match std::fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .any(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("MapTex1_") && name.ends_with(".sbmaptex")
            }),
        Err(_) => false,
    }
}

/// 在指定根目录下递归或多候选路径查找 MapTex/MainField 目录
pub fn find_maptex_in_directory(root: &Path, max_depth: usize) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }

    // 1. 常见固定相对子路径直查
    let standard_candidates = [
        root.join("content").join("UI").join("MapTex").join("MainField"),
        root.join("UI").join("MapTex").join("MainField"),
        root.join("romfs").join("UI").join("MapTex").join("MainField"),
        root.join("01007EF00011E000").join("romfs").join("UI").join("MapTex").join("MainField"),
        root.join("MainField"),
    ];
    for cand in &standard_candidates {
        if is_valid_mainfield_dir(cand) {
            return Some(cand.clone());
        }
    }

    // 2. 深度限制递归遍历查找 (防止深层扫描耗时)
    if max_depth > 0 {
        find_maptex_recursive(root, 0, max_depth)
    } else {
        None
    }
}

/// 递归搜索辅助函数
fn find_maptex_recursive(dir: &Path, current_depth: usize, max_depth: usize) -> Option<PathBuf> {
    if current_depth >= max_depth {
        return None;
    }

    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name.eq_ignore_ascii_case("MainField") {
            if is_valid_mainfield_dir(&path) {
                return Some(path);
            }
        } else if !name.starts_with('.') && name != "target" && name != "node_modules" {
            if let Some(found) = find_maptex_recursive(&path, current_depth + 1, max_depth) {
                return Some(found);
            }
        }
    }
    None
}

/// 收集指定平台在偏好设置与衍生环境下的候选查找目录
fn collect_platform_maptex_candidates(
    plat: crate::model::PlatformType,
    gp: &crate::model::GamePathSettings,
) -> Vec<PathBuf> {
    let mut list = Vec::new();
    match plat {
        crate::model::PlatformType::Switch => {
            if !gp.switch.game_dir.trim().is_empty() {
                list.push(PathBuf::from(gp.switch.game_dir.trim()));
            }
            if !gp.switch.dlc_dir.trim().is_empty() {
                let dlc = gp.switch.dlc_dir.trim();
                list.push(PathBuf::from(dlc));
                if dlc.contains("01007EF00011F001") {
                    list.push(PathBuf::from(dlc.replace("01007EF00011F001", "01007EF00011E000")));
                }
            }
            list.push(PathBuf::from(r"C:\Game\BOTW\01007EF00011E000\romfs"));
            list.push(PathBuf::from(r"C:\Game\BOTW\01007EF00011E000\romfs\UI\MapTex\MainField"));
        }
        crate::model::PlatformType::WiiU => {
            if !gp.wiiu.update_dir.trim().is_empty() {
                list.push(PathBuf::from(gp.wiiu.update_dir.trim()));
            }
            if !gp.wiiu.game_dir.trim().is_empty() {
                list.push(PathBuf::from(gp.wiiu.game_dir.trim()));
            }
            list.push(PathBuf::from(r"E:\CEMU2.6\Games\The Legend of Zelda-Breath of the Wild\content\UI\MapTex\MainField"));
            list.push(PathBuf::from(r"E:\Wiiu emu\Botw_Dump\Games\The Legend of Zelda-Breath of the Wild\content\UI\MapTex\MainField"));
        }
    }
    list
}

/// 解析地图瓦片源文件目录
/// 返回值：Some((源路径, 是否来自已打开项目))
pub fn resolve_maptex_source(
    project_path: Option<&str>,
    game_paths: Option<&crate::model::GamePathSettings>,
) -> Option<(PathBuf, bool)> {
    // 1. 优先在当前已打开项目中检测
    if let Some(proj) = project_path {
        let p = Path::new(proj);
        if let Some(src) = find_maptex_in_directory(p, 4) {
            return Some((src, true));
        }
    }

    // 2. 若项目中未包含，回退检查设置中的游戏 Dump 路径 (严格优先匹配当前选择的平台)
    if let Some(gp) = game_paths {
        let (primary_plat, fallback_plat) = match gp.current_platform {
            crate::model::PlatformType::Switch => (crate::model::PlatformType::Switch, crate::model::PlatformType::WiiU),
            crate::model::PlatformType::WiiU => (crate::model::PlatformType::WiiU, crate::model::PlatformType::Switch),
        };

        // 优先在用户当前所选平台中查找
        for cand in collect_platform_maptex_candidates(primary_plat, gp) {
            if is_valid_mainfield_dir(&cand) {
                return Some((cand, false));
            }
            if let Some(src) = find_maptex_in_directory(&cand, 3) {
                return Some((src, false));
            }
        }

        // 当前平台未找到时，才检查备选平台
        for cand in collect_platform_maptex_candidates(fallback_plat, gp) {
            if is_valid_mainfield_dir(&cand) {
                return Some((cand, false));
            }
            if let Some(src) = find_maptex_in_directory(&cand, 3) {
                return Some((src, false));
            }
        }
    }

    None
}

/// 统计目录中 .png 瓦片数量
#[allow(dead_code)]
pub fn count_pngs_in_dir(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("png"))
                    .unwrap_or(false)
            })
            .count(),
        Err(_) => 0,
    }
}

/// 统计当前生效的瓦片数量 (项目专有瓦片优先)
#[allow(dead_code)]
pub fn count_cached_map_tiles(project_path: Option<&str>) -> usize {
    if let Some(proj) = project_path {
        let p_dir = get_project_tiles_cache_dir(proj);
        let cnt = count_pngs_in_dir(&p_dir);
        if cnt > 0 {
            return cnt;
        }
    }
    count_pngs_in_dir(&get_global_map_tiles_cache_dir())
}

/// 获取指定分区编号的瓦片物理文件路径 (项目专有缓存优先，全局基准回退)
pub fn get_tile_path(c_char: char, r_num: i32, project_path: Option<&str>) -> Option<PathBuf> {
    let filename = format!("{}-{}.png", c_char, r_num);

    // 1. 项目专有瓦片
    if let Some(proj) = project_path {
        let p_tile = get_project_tiles_cache_dir(proj).join(&filename);
        if p_tile.exists() {
            return Some(p_tile);
        }
    }

    // 2. 全局基准瓦片
    let g_tile = get_global_map_tiles_cache_dir().join(&filename);
    if g_tile.exists() {
        return Some(g_tile);
    }

    None
}

/// 执行瓦片批量提取 (纯 Rust 原生多线程高速处理)
pub fn execute_tile_extraction(src_dir: &Path, out_dir: &Path) -> Result<usize, String> {
    let count = botw_map::extract_tiles_batch(src_dir, out_dir)
        .map_err(|e| format!("瓦片提取出错: {}", e))?;

    if count == 0 {
        return Err("提取完成但未能输出有效的 PNG 瓦片".to_string());
    }

    Ok(count)
}

/// 将分区列名转为海拉鲁世界 X 坐标起点
pub fn col_to_world_x(col: char) -> Option<f32> {
    match col {
        'Z' => Some(-6000.0),
        'A' => Some(-5000.0),
        'B' => Some(-4000.0),
        'C' => Some(-3000.0),
        'D' => Some(-2000.0),
        'E' => Some(-1000.0),
        'F' => Some(0.0),
        'G' => Some(1000.0),
        'H' => Some(2000.0),
        'I' => Some(3000.0),
        'J' => Some(4000.0),
        'K' => Some(5000.0),
        _ => None,
    }
}

static UNIFIED_MAP_CACHE: std::sync::Mutex<Option<(Option<String>, Option<PathBuf>)>> =
    std::sync::Mutex::new(None);

/// 失效全景整图路径缓存 (提取新瓦片后调用)
#[allow(dead_code)]
pub fn invalidate_unified_world_map_cache() {
    if let Ok(mut guard) = UNIFIED_MAP_CACHE.lock() {
        *guard = None;
    }
}

/// 获取统一拼接的海拉鲁全景整图路径 (带内存缓存，项目专有优先，全局缓存回退)
pub fn get_unified_world_map_path(project_path: Option<&str>) -> Option<PathBuf> {
    let proj_key = project_path.map(|s| s.to_string());
    if let Ok(guard) = UNIFIED_MAP_CACHE.lock() {
        if let Some((ref cached_proj, ref path)) = *guard {
            if *cached_proj == proj_key {
                return path.clone();
            }
        }
    }

    let calculated = get_unified_world_map_path_uncached(project_path);
    if let Ok(mut guard) = UNIFIED_MAP_CACHE.lock() {
        *guard = Some((proj_key, calculated.clone()));
    }
    calculated
}

fn get_unified_world_map_path_uncached(project_path: Option<&str>) -> Option<PathBuf> {
    if let Some(proj) = project_path {
        let p_map = get_project_tiles_cache_dir(proj).join("world_map_full.png");
        if p_map.exists() {
            return Some(p_map);
        }
    }
    let g_map = get_global_map_tiles_cache_dir().join("world_map_full.png");
    if g_map.exists() {
        return Some(g_map);
    }
    None
}

/// 渲染官方真实海拉鲁卫星地图图层 (对齐 ice-spear：统一整图单 Render Pass 绘制，零瓦片重叠)
pub fn render_tiles_layer(pan: (f32, f32), zoom: f32, project_path: Option<&str>) -> impl IntoElement {
    // 方案对齐 ice-spear：
    // 在 ice-spear 中，120 个瓦片载入单个连续纹理并在同一个 Render Pass 中绘制，无缝且零重叠。
    // 此处优先检测并使用 120 个瓦片拼合而成的统一海拉鲁全景大图 (world_map_full.png)。
    if let Some(unified_map) = get_unified_world_map_path(project_path) {
        // 海拉鲁大陆世界坐标范围：X: -6000.0 ~ 6000.0 (宽 12000), Z: -5000.0 ~ 5000.0 (高 10000)
        let sx = pan.0 - 6000.0 * zoom;
        let sy = pan.1 - 5000.0 * zoom;
        let w = 12000.0 * zoom;
        let h = 10000.0 * zoom;

        // 视口边界快速裁剪
        if sx + w > -50.0 && sx < 3500.0 && sy + h > -50.0 && sy < 2500.0 {
            return div()
                .absolute()
                .inset_0()
                .child(
                    img(unified_map)
                        .id("unified_hyrule_world_map")
                        .absolute()
                        .left(px(sx))
                        .top(px(sy))
                        .w(px(w))
                        .h(px(h))
                        .object_fit(ObjectFit::Fill),
                )
                .into_any_element();
        }
    }

    // 备用降级路径 (当整图尚在生成中时，按网格紧密连续绘制，严丝合缝，零重叠)
    let proj_owned = project_path.map(|s| s.to_string());
    let cols = ['Z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K'];

    div()
        .absolute()
        .inset_0()
        .children(cols.iter().filter_map(move |&c_char| {
            let world_x = col_to_world_x(c_char)?;
            let proj_ref = proj_owned.clone();
            Some((0..=9).filter_map(move |r_num| {
                let world_z = -5000.0 + (r_num as f32) * 1000.0;
                let next_world_x = world_x + 1000.0;
                let next_world_z = world_z + 1000.0;

                let sx = pan.0 + world_x * zoom;
                let sy = pan.1 + world_z * zoom;
                let next_sx = pan.0 + next_world_x * zoom;
                let next_sy = pan.1 + next_world_z * zoom;
                let w = next_sx - sx;
                let h = next_sy - sy;

                if next_sx < -100.0 || sx > 3500.0 || next_sy < -100.0 || sy > 2500.0 {
                    return None;
                }

                let tile_path = get_tile_path(c_char, r_num, proj_ref.as_deref())?;

                Some(
                    img(tile_path)
                        .id(SharedString::from(format!("tile_{}-{}", c_char, r_num)))
                        .absolute()
                        .left(px(sx))
                        .top(px(sy))
                        .w(px(w))
                        .h(px(h))
                        .object_fit(ObjectFit::Fill),
                )
            }))
        }).flatten())
        .into_any_element()
}
