use std::fs;
use std::path::Path;
use gpui::*;
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 异步打开文件选择器并替换 SARC 封包内的目标文件
pub fn pick_and_replace_file(
    model: &Entity<AppState>,
    target_path: String,
    name: String,
    cx: &mut App,
) {
    model.update(cx, |st, cx| {
        st.close_file_tree_context_menu();
        cx.notify();
    });

    let model = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .set_title("替换文件")
            .pick_file()
            .await;

        if let Some(file) = handle {
            let picked_path = file.path().to_path_buf();
            let result = fs::read(&picked_path)
                .map_err(|e| e.to_string())
                .and_then(|bytes| crate::model::write_file_to_target(&target_path, &bytes));

            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    match result {
                        Ok(()) => {
                            let is_open = st.open_tabs.iter().any(|t| t.file_path == target_path);
                            if is_open {
                                st.load_file_by_path(&target_path);
                            }
                            st.reload_project();
                            st.toast = Some(Toast::success(format!("{} 「{}」", st.t("toast.replace_success"), name)));
                        }
                        Err(e) => {
                            st.toast = Some(Toast::error(format!("{}: {}", st.t("toast.replace_error"), e)));
                        }
                    }
                    cx.notify();
                });
            });
        }
    })
    .detach();
}

/// 异步打开多文件选择器并批量添加导入至指定的容器中
pub fn pick_and_add_files(
    model: &Entity<AppState>,
    container_path: String,
    is_dir: bool,
    is_sarc: bool,
    cx: &mut App,
) {
    model.update(cx, |st, cx| {
        st.close_file_tree_context_menu();
        cx.notify();
    });

    let model = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .set_title("添加文件")
            .pick_files()
            .await;

        if let Some(files) = handle {
            let mut added_count = 0;
            let mut last_error = None;

            for file in files {
                let file_path = file.path().to_path_buf();
                let file_name = match file_path.file_name().and_then(|n| n.to_str()) {
                    Some(name) => name.to_string(),
                    None => continue,
                };

                let bytes = match fs::read(&file_path) {
                    Ok(b) => b,
                    Err(e) => {
                        last_error = Some(e.to_string());
                        continue;
                    }
                };

                let dest_path = if container_path.starts_with("SARC:") {
                    if is_sarc && !container_path.contains("//") {
                        format!("{}//{}", container_path, file_name)
                    } else if is_dir {
                        format!("{}/{}", container_path.trim_end_matches('/'), file_name)
                    } else {
                        if let Some(idx) = container_path.rfind('/') {
                            format!("{}/{}", &container_path[..idx], file_name)
                        } else if let Some(idx) = container_path.rfind("//") {
                            format!("{}/{}", &container_path[..idx + 2], file_name)
                        } else {
                            format!("{}//{}", container_path, file_name)
                        }
                    }
                } else {
                    let p = Path::new(&container_path);
                    if is_dir {
                        p.join(&file_name).to_string_lossy().to_string()
                    } else {
                        let parent = p.parent().unwrap_or(p);
                        parent.join(&file_name).to_string_lossy().to_string()
                    }
                };

                match crate::model::write_file_to_target(&dest_path, &bytes) {
                    Ok(()) => added_count += 1,
                    Err(e) => last_error = Some(e),
                }
            }

            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    st.reload_project();
                    if added_count > 0 {
                        st.toast = Some(Toast::success(format!("{} ({} 个文件)", st.t("toast.add_success"), added_count)));
                    } else if let Some(err) = last_error {
                        st.toast = Some(Toast::error(format!("{}: {}", st.t("toast.add_error"), err)));
                    }
                    cx.notify();
                });
            });
        }
    })
    .detach();
}

/// 异步打开保存文件对话框并导出单文件至本地磁盘
pub fn pick_and_export_file(
    model: &Entity<AppState>,
    path: String,
    default_name: String,
    cx: &mut App,
) {
    model.update(cx, |st, cx| {
        st.close_file_tree_context_menu();
        cx.notify();
    });

    let model = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .set_file_name(&default_name)
            .save_file()
            .await;

        if let Some(file) = handle {
            let dest_path = file.path().to_path_buf();
            let result = crate::model::file_tree::read_file_from_path(&path)
                .and_then(|bytes| fs::write(&dest_path, bytes).map_err(|e| e.to_string()));

            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    match result {
                        Ok(()) => {
                            let name = dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            st.toast = Some(Toast::success(format!("{} -> {}", st.t("toast.export_success"), name)));
                        }
                        Err(e) => {
                            st.toast = Some(Toast::error(format!("{}: {}", st.t("toast.export_error"), e)));
                        }
                    }
                    cx.notify();
                });
            });
        }
    })
    .detach();
}

/// 异步打开目录选择对话框并解包导出文件夹或 SARC 封包至本地磁盘
pub fn pick_and_export_container(
    model: &Entity<AppState>,
    path: String,
    is_dir: bool,
    is_sarc: bool,
    cx: &mut App,
) {
    model.update(cx, |st, cx| {
        st.close_file_tree_context_menu();
        cx.notify();
    });

    let model = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .pick_folder()
            .await;

        if let Some(folder) = handle {
            let dest_dir = folder.path().to_path_buf();
            let is_sarc_target = path.starts_with("SARC:") || is_sarc;
            let result = if is_sarc_target {
                crate::model::extract_sarc_to_dir(&path, &dest_dir)
            } else if is_dir {
                crate::model::copy_dir_all(Path::new(&path), &dest_dir)
            } else {
                Err("目标不是有效的文件夹或封包".to_string())
            };

            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    match result {
                        Ok(count) => {
                            st.toast = Some(Toast::success(format!("{} ({} 个文件)", st.t("toast.export_success"), count)));
                        }
                        Err(e) => {
                            st.toast = Some(Toast::error(format!("{}: {}", st.t("toast.export_error"), e)));
                        }
                    }
                    cx.notify();
                });
            });
        }
    })
    .detach();
}
