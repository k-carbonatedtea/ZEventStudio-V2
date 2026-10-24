use gpui::*;
use std::path::{Path, PathBuf};
use std::process::Command;

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

    // 2. 若项目中未包含，回退检查设置中的游戏 Dump 路径
    if let Some(gp) = game_paths {
        let settings_candidates = [
            PathBuf::from(&gp.wiiu.game_dir),
            PathBuf::from(&gp.wiiu.update_dir),
            PathBuf::from(&gp.switch.game_dir),
        ];
        for cand in &settings_candidates {
            if !cand.as_os_str().is_empty() {
                if let Some(src) = find_maptex_in_directory(cand, 3) {
                    return Some((src, false));
                }
            }
        }
    }

    // 3. 最终回退：系统已知典型 Dump 路径
    let known_paths = [
        PathBuf::from(r"E:\CEMU2.6\Games\The Legend of Zelda-Breath of the Wild\content\UI\MapTex\MainField"),
        PathBuf::from(r"E:\Wiiu emu\Botw_Dump\Games\The Legend of Zelda-Breath of the Wild\content\UI\MapTex\MainField"),
        PathBuf::from(r"C:\Game\BOTW\01007EF00011E000\romfs\UI\MapTex\MainField"),
    ];
    for p in &known_paths {
        if is_valid_mainfield_dir(p) {
            return Some((p.clone(), false));
        }
    }

    None
}

/// 统计目录中 .png 瓦片数量
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

/// 执行瓦片批量提取脚本 (在后台工作线程中同步阻塞运行)
pub fn execute_tile_extraction(src_dir: &Path, out_dir: &Path) -> Result<usize, String> {
    let script_candidates = [
        PathBuf::from(r"C:\Users\Administrator\Desktop\Zevent\ZEventStudio\scripts\extract_tiles.py"),
        PathBuf::from("scripts/extract_tiles.py"),
    ];

    let script_path = script_candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .ok_or_else(|| "未找到瓦片提取脚本 scripts/extract_tiles.py".to_string())?;

    let src_str = src_dir.to_string_lossy().to_string();
    let out_str = out_dir.to_string_lossy().to_string();

    let output = Command::new("python")
        .arg(&script_path)
        .arg(&src_str)
        .arg(&out_str)
        .output()
        .map_err(|e| format!("启动 Python 进程失败: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("提取脚本执行出错: {}", err));
    }

    let count = count_pngs_in_dir(out_dir);
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

/// 渲染官方真实海拉鲁卫星地图瓦片图层 (支持项目专有瓦片与视口裁剪 Viewport Culling)
pub fn render_tiles_layer(pan: (f32, f32), zoom: f32, project_path: Option<&str>) -> impl IntoElement {
    let proj_owned = project_path.map(|s| s.to_string());
    let cols = ['Z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K'];

    div()
        .absolute()
        .inset_0()
        .children(cols.iter().filter_map(move |&c_char| {
            let world_x = col_to_world_x(c_char)?;
            let proj_ref = proj_owned.clone();
            Some((0..=9).filter_map(move |r_num| {
                // 计算当前分区在世界坐标系中的左上角点 (Section 尺寸为 1000x1000)
                let world_z = -5000.0 + (r_num as f32) * 1000.0;

                // 投影至画布相对像素坐标
                let sx = pan.0 + world_x * zoom;
                let sy = pan.1 + world_z * zoom;
                let tile_size = 1000.0 * zoom;

                // 视口边界快速裁剪 (剔除可视区域外瓦片，保障 60+ FPS)
                if sx + tile_size < -50.0 || sx > 3000.0 || sy + tile_size < -50.0 || sy > 2000.0 {
                    return None;
                }

                let tile_path = get_tile_path(c_char, r_num, proj_ref.as_deref())?;

                Some(
                    div()
                        .id(SharedString::from(format!("tile_{}-{}", c_char, r_num)))
                        .absolute()
                        .left(px(sx))
                        .top(px(sy))
                        .size(px(tile_size))
                        .overflow_hidden()
                        .child(img(tile_path).size_full()),
                )
            }))
        }).flatten())
}
