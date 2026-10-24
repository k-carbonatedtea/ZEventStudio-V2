use std::path::{Path, PathBuf};
use gpui::*;
use gpui_component::tooltip::Tooltip;
use crate::model::settings::{detect_platform_from_path, PlatformType};
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use crate::ui::state_settings::SettingsTab;
use crate::ui::views::quest_map::tiles_layer::{get_global_map_tiles_cache_dir, resolve_maptex_source};

/// 素材来源类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetOrigin {
    Project,
    GameDump,
    BuiltinCache,
    NotConfigured,
}

/// 素材来源与平台判定结果
pub struct AssetSourceInfo {
    pub platform: PlatformType,
    pub origin: AssetOrigin,
    pub full_path: String,
    pub display_path: String,
}

/// 解析当前海拉鲁地图素材的来源路径与目标平台
pub fn resolve_asset_source_info(state: &AppState) -> AssetSourceInfo {
    // 1. 优先尝试解析瓦片纹理物理目录
    if let Some((maptex_dir, is_proj)) = resolve_maptex_source(state.project_path.as_deref(), Some(&state.game_paths)) {
        let full = maptex_dir.to_string_lossy().to_string();
        let detected = detect_platform_from_path(&full).unwrap_or(state.game_paths.current_platform);
        let origin = if is_proj { AssetOrigin::Project } else { AssetOrigin::GameDump };
        let short = compact_display_path(&full, if is_proj { 22 } else { 26 });
        return AssetSourceInfo {
            platform: detected,
            origin,
            full_path: full,
            display_path: short,
        };
    }

    // 2. 若瓦片未解压但当前打开了 Mod 项目，优先归属于项目
    if let Some(ref proj) = state.project_path {
        let detected = detect_platform_from_path(proj).unwrap_or(state.game_paths.current_platform);
        let short = compact_display_path(proj, 22);
        return AssetSourceInfo {
            platform: detected,
            origin: AssetOrigin::Project,
            full_path: proj.clone(),
            display_path: short,
        };
    }

    // 3. 检查全局配置的游戏路径 (根据当前平台优先匹配)
    let gp = &state.game_paths;
    let (chosen_path, chosen_plat) = match gp.current_platform {
        PlatformType::Switch => {
            if !gp.switch.game_dir.trim().is_empty() {
                (gp.switch.game_dir.clone(), PlatformType::Switch)
            } else if !gp.wiiu.game_dir.trim().is_empty() {
                (gp.wiiu.game_dir.clone(), PlatformType::WiiU)
            } else if !gp.wiiu.update_dir.trim().is_empty() {
                (gp.wiiu.update_dir.clone(), PlatformType::WiiU)
            } else {
                (String::new(), PlatformType::Switch)
            }
        }
        PlatformType::WiiU => {
            if !gp.wiiu.update_dir.trim().is_empty() {
                (gp.wiiu.update_dir.clone(), PlatformType::WiiU)
            } else if !gp.wiiu.game_dir.trim().is_empty() {
                (gp.wiiu.game_dir.clone(), PlatformType::WiiU)
            } else if !gp.switch.game_dir.trim().is_empty() {
                (gp.switch.game_dir.clone(), PlatformType::Switch)
            } else {
                (String::new(), PlatformType::WiiU)
            }
        }
    };

    if !chosen_path.is_empty() {
        let short = compact_display_path(&chosen_path, 26);
        return AssetSourceInfo {
            platform: chosen_plat,
            origin: AssetOrigin::GameDump,
            full_path: chosen_path,
            display_path: short,
        };
    }

    // 4. 回退至应用内建全局瓦片缓存
    let cache_dir = get_global_map_tiles_cache_dir();
    let full = cache_dir.to_string_lossy().to_string();
    let short = compact_display_path(&full, 24);
    AssetSourceInfo {
        platform: gp.current_platform,
        origin: if cache_dir.exists() { AssetOrigin::BuiltinCache } else { AssetOrigin::NotConfigured },
        full_path: full,
        display_path: short,
    }
}

/// 智能精简路径用于单行窄宽展示
fn compact_display_path(path: &str, max_len: usize) -> String {
    if path.chars().count() <= max_len {
        return path.to_string();
    }
    let p = Path::new(path);
    let leaf = p.file_name().and_then(|f| f.to_str()).unwrap_or("");
    let parent = p.parent().and_then(|pr| pr.file_name()).and_then(|f| f.to_str()).unwrap_or("");

    let prefix = p.components().next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_default();

    if !parent.is_empty() && !leaf.is_empty() {
        let tail = format!(r"{}\{}", parent, leaf);
        if !prefix.is_empty() {
            format!(r"{}\...\{}", prefix.trim_end_matches('\\'), tail)
        } else {
            format!(r"...\ {}", tail)
        }
    } else if !leaf.is_empty() {
        format!(r"...\ {}", leaf)
    } else {
        path.chars().take(max_len).collect::<String>() + "..."
    }
}

/// 渲染 2D 地图顶部 HUD 中的素材来源与平台指示器
pub fn render_asset_source_indicator(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let info = resolve_asset_source_info(state);

    let is_switch = info.platform.is_switch();
    let (plat_bg, plat_border, plat_fg, plat_text_key, plat_icon) = if is_switch {
        (
            rgba(0xe6001222),
            rgba(0xe6001266),
            rgb(0xff5566),
            "quests.platform_switch",
            "🎮",
        )
    } else {
        (
            rgba(0x009acd22),
            rgba(0x009acd66),
            rgb(0x38bdf8),
            "quests.platform_wiiu",
            "🖥️",
        )
    };

    let (origin_icon, origin_label_key) = match info.origin {
        AssetOrigin::Project => ("📦", "quests.source_project"),
        AssetOrigin::GameDump => ("📂", "quests.source_game_dump"),
        AssetOrigin::BuiltinCache => ("⚡", "quests.source_builtin_cache"),
        AssetOrigin::NotConfigured => ("⚠️", "quests.source_not_configured"),
    };

    let model_plat = model.clone();
    let model_path = model.clone();
    let target_plat = info.platform;
    let full_path = info.full_path.clone();
    let has_path = info.origin != AssetOrigin::NotConfigured && !info.full_path.is_empty();

    let tip_plat = format!("{} ({})", state.t(plat_text_key), state.t("quests.source_click_settings"));
    let tip_full_path = full_path.clone();
    let tip_origin_name = state.t(origin_label_key).to_string();
    let tip_click_hint = state.t("quests.source_click_open_explorer").to_string();

    div()
        .id("map_asset_source_indicator")
        .flex()
        .items_center()
        .gap_1p5()
        .px_2()
        .py_0p5()
        .rounded_md()
        .bg(pal.card_surface.opacity(0.7))
        .border_1()
        .border_color(pal.panel_border)
        // 1. 平台标签 Badge (Switch / Wii U，点击直达对应平台设置)
        .child(
            div()
                .id("badge_platform_switch_wiiu")
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1()
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .bg(plat_bg)
                .border_1()
                .border_color(plat_border)
                .hover(|s| s.bg(plat_bg.opacity(1.5)))
                .tooltip(move |window, cx| {
                    let p = tip_plat.clone();
                    Tooltip::element(move |_window, _cx| {
                        div().text_size(px(10.5)).child(p.clone())
                    })
                    .build(window, cx)
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_plat.update(cx, |st, cx| {
                        st.open_settings(match target_plat {
                            PlatformType::Switch => SettingsTab::Switch,
                            PlatformType::WiiU => SettingsTab::WiiU,
                        });
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .text_size(px(10.0))
                        .child(plat_icon)
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(plat_fg)
                        .child(state.t(plat_text_key))
                )
        )
        // 2. 素材获取物理路径胶囊 (显示截短路径，点击在资源管理器中打开，悬停查看完整路径)
        .child(
            div()
                .id("badge_asset_source_path")
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1p5()
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .bg(pal.card_hover.opacity(0.5))
                .border_1()
                .border_color(pal.panel_border.opacity(0.6))
                .hover(|s| s.bg(pal.card_hover).border_color(pal.primary_accent.opacity(0.5)))
                .tooltip(move |window, cx| {
                    let fp = tip_full_path.clone();
                    let og = tip_origin_name.clone();
                    let ch = tip_click_hint.clone();
                    Tooltip::element(move |_window, _cx| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .text_size(px(10.5))
                            .child(format!("[{}] {}", og, fp))
                            .child(div().text_size(px(9.5)).text_color(rgb(0x9ca3af)).child(ch.clone()))
                    })
                    .build(window, cx)
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    if has_path {
                        let p = PathBuf::from(&full_path);
                        if p.exists() {
                            let _ = std::process::Command::new("explorer").arg(&full_path).spawn();
                            model_path.update(cx, |st, cx| {
                                st.toast = Some(Toast::info(st.t("quests.source_opened_toast")));
                                cx.notify();
                            });
                            return;
                        }
                    }
                    model_path.update(cx, |st, cx| {
                        st.open_settings(match target_plat {
                            PlatformType::Switch => SettingsTab::Switch,
                            PlatformType::WiiU => SettingsTab::WiiU,
                        });
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .text_size(px(10.5))
                        .child(origin_icon)
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(pal.text_secondary)
                        .child(format!("[{}]", state.t(origin_label_key)))
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(pal.text_primary)
                        .child(info.display_path.clone())
                )
        )
}
