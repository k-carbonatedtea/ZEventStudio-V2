use gpui::*;
use crate::ui::components::Toast;
use crate::ui::state::{AppState, AppWorkspaceTab, RecentProjectItem};

#[allow(dead_code)]
/// 渲染欢迎启动页的「快速开始」功能面板
pub fn render_quick_start_tab(

    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2p5()
        // 1. 打开单个文件
        .child(render_action_card(
            "📁",
            state.t("empty.openFile"),
            state.t("empty.openFileDesc"),
            Some("Ctrl+O"),
            hsla(215.0 / 360.0, 0.70, 0.40, 1.0),
            {
                let model = model.clone();
                move |cx| {
                    pick_and_load_file(&model, cx);
                }
            },
        ))
        // 2. 打开 Mod 文件夹
        .child(render_action_card(
            "🗂️",
            state.t("empty.openMod"),
            state.t("empty.openModDesc"),
            None,
            hsla(185.0 / 360.0, 0.70, 0.35, 1.0),
            {
                let model = model.clone();
                move |cx| {
                    pick_and_load_mod_folder(&model, cx);
                }
            },
        ))
        // 3. 新建空白事件流
        .child(render_action_card(
            "📄",
            state.t("empty.newFlow"),
            state.t("empty.newFlowDesc"),
            Some("Ctrl+N"),
            hsla(145.0 / 360.0, 0.65, 0.35, 1.0),
            {
                let model = model.clone();
                move |cx| {
                    model.update(cx, |st, cx| {
                        st.new_blank_flow();
                        cx.notify();
                    });
                }
            },
        ))
}

fn render_action_card<F>(
    icon_str: &'static str,
    title: &str,
    desc: &str,
    shortcut: Option<&'static str>,
    icon_bg: Hsla,
    on_click: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let title_str = title.to_string();
    let desc_str = desc.to_string();

    div()
        .px_3p5()
        .py_3()
        .rounded_lg()
        .bg(hsla(220.0 / 360.0, 0.35, 0.13, 0.95))
        .border_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.22, 1.0))
        .hover(|s| {
            s.bg(hsla(215.0 / 360.0, 0.40, 0.18, 0.98))
                .border_color(hsla(215.0 / 360.0, 0.60, 0.45, 1.0))
        })
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                // 左侧彩色大图标
                .child(
                    div()
                        .w(px(36.0))
                        .h(px(36.0))
                        .rounded_md()
                        .bg(icon_bg)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(18.0))
                        .child(icon_str),
                )
                // 中间标题与说明
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(hsla(0.0, 0.0, 1.0, 0.95))
                                .child(title_str),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.60, 1.0))
                                .child(desc_str),
                        ),
                ),
        )
        // 右侧快捷键胶囊
        .children(shortcut.map(|sc| {
            div()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(hsla(220.0 / 360.0, 0.40, 0.08, 1.0))
                .border_1()
                .border_color(hsla(220.0 / 360.0, 0.25, 0.20, 1.0))
                .text_size(px(10.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(hsla(215.0 / 360.0, 0.20, 0.70, 1.0))
                .child(sc)
        }))
}

/// 弹出原生文件选择器并读取
pub fn pick_and_load_file(model: &Entity<AppState>, cx: &mut App) {
    let model = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .add_filter(
                "EventFlow & Zelda Packages (*.bfevfl, *.sbeventpack, *.sbactorpack, *.pack, *.msbt, *.json)",
                &["bfevfl", "sbeventpack", "sbactorpack", "pack", "sarc", "ssarc", "spack", "msbt", "aamp", "byml", "json"],
            )
            .set_title("选择 BotW 游戏事件流或封包文件")
            .pick_file()
            .await;

        if let Some(file) = handle {
            let path = file.path().to_path_buf();
            let path_str = path.to_string_lossy().to_string();
            let file_name = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown.bfevfl".to_string());

            let is_sarc = crate::model::is_sarc_extension(&file_name);
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    if is_sarc {
                        st.open_pack_file(&path_str);
                        cx.notify();
                        return;
                    }

                    if ext == "bfevfl" {
                        if let Ok(bytes) = std::fs::read(&path) {
                            if let Ok(mut loaded_graph) =
                                crate::model::evfl::load_bfevfl_from_bytes(&bytes)
                            {
                                loaded_graph.name = file_name.clone();
                                loaded_graph.ensure_actors_populated();
                                crate::model::layout::auto_layout_graph(&mut loaded_graph);
                                st.graph = loaded_graph;
                                st.fit_view();
                                st.current_tab = AppWorkspaceTab::Flowchart;
                                st.toast = Some(Toast::success(format!("📂 成功加载: {}", file_name)));
                                st.recent_projects.insert(
                                    0,
                                    RecentProjectItem {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        name: file_name.clone(),
                                        path: path_str.clone(),
                                        project_type: ext.clone(),
                                    },
                                );
                                cx.notify();
                                return;
                            }
                        }
                    }

                    st.current_tab = AppWorkspaceTab::Flowchart;
                    st.toast = Some(Toast::success(format!("📂 已打开文件: {}", file_name)));
                    st.recent_projects.insert(
                        0,
                        RecentProjectItem {
                            id: uuid::Uuid::new_v4().to_string(),
                            name: file_name,
                            path: path_str,
                            project_type: ext,
                        },
                    );
                    cx.notify();
                });
            });
        }
    })
    .detach();
}

/// 弹出原生封包选择器 (.sbeventpack, .sbactorpack, .pack, .sarc, .ssarc, .spack)
pub fn pick_and_load_pack_file(model: &Entity<AppState>, cx: &mut App) {
    let model = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .add_filter(
                "Nintendo SARC Archives (*.sbeventpack, *.sbactorpack, *.pack, *.sarc, *.ssarc, *.spack)",
                &["sbeventpack", "sbactorpack", "pack", "sarc", "ssarc", "spack", "bactorpack"],
            )
            .set_title("选择 BotW SARC 封包文件")
            .pick_file()
            .await;

        if let Some(file) = handle {
            let path_str = file.path().to_string_lossy().to_string();
            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    st.open_pack_file(&path_str);
                    cx.notify();
                });
            });
        }
    })
    .detach();
}

/// 弹出原生目录选择器
pub fn pick_and_load_mod_folder(model: &Entity<AppState>, cx: &mut App) {
    let model = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .set_title("选择 Wii U (content) 或 Switch (romfs) 模组根目录")
            .pick_folder()
            .await;

        if let Some(folder) = handle {
            let path_str = folder.path().to_string_lossy().to_string();
            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    st.open_project_directory(&path_str);
                    cx.notify();
                });
            });
        }
    })
    .detach();
}
