//! UMii 编辑器文件导入与导出操作组件 (Mii导入、BUMII导出、YAML导出)

use gpui::*;
use crate::model::umii::collect_model_search_dirs;
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;

/// 渲染导入外部 Mii / UMii 文件按钮
pub fn render_import_button(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tid = tab_id.to_string();
    let model_cb = model.clone();
    div()
        .id("btn_import_mii")
        .flex()
        .items_center()
        .gap_1()
        .px_2p5()
        .py_1()
        .rounded_md()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let tid = tid.clone();
            let model_cb = model_cb.clone();
            cx.spawn(async move |cx| {
                if let Some(file_path) = rfd::AsyncFileDialog::new()
                    .add_filter("Mii / Avatar Files", &["mii", "mae", "ffsd", "cfsd", "charinfo", "bumii"])
                    .pick_file()
                    .await
                {
                    let bytes = file_path.read().await;
                    let name = file_path.file_name();
                    cx.update(|cx| {
                        model_cb.update(cx, |st, cx| {
                            if let Some(tab) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                if let Some(ref mut u) = tab.umii_state {
                                    match crate::ui::state_umii::parse_umii_data_from_bytes(&bytes, &name) {
                                        Ok(new_data) => {
                                            u.data = new_data;
                                            tab.is_modified = true;
                                            let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                            let _ = u.synthesize_model(&dirs);
                                            st.toast = Some(Toast::success(st.t("umii.import_success")));
                                        }
                                        Err(e) => {
                                            st.toast = Some(Toast::error(format!("{}: {}", st.t("umii.import_error"), e)));
                                        }
                                    }
                                }
                            }
                            cx.notify();
                        });
                    });
                }
            }).detach();
        })
        .child(div().text_xs().text_color(pal.text_primary).child(format!("📥 {}", state.t("umii.import_mii"))))
}

/// 渲染导出二进制 BUMII 文件按钮
pub fn render_export_bumii_button(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tid = tab_id.to_string();
    let model_cb = model.clone();
    div()
        .id("btn_export_bumii")
        .flex()
        .items_center()
        .gap_1()
        .px_2p5()
        .py_1()
        .rounded_md()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let tid = tid.clone();
            let model_cb = model_cb.clone();
            cx.spawn(async move |cx| {
                let opt_bytes: Option<Vec<u8>> = cx.update(|cx| {
                    model_cb.read(cx).open_tabs.iter()
                        .find(|t| t.id == tid)
                        .and_then(|t| t.umii_state.as_ref())
                        .and_then(|u| u.data.to_binary().ok())
                });
                let Some(bytes) = opt_bytes else { return; };
                if let Some(file_handle) = rfd::AsyncFileDialog::new()
                    .set_file_name("custom_npc.bumii")
                    .add_filter("BotW UMii Binary", &["bumii"])
                    .save_file()
                    .await
                {
                    let _ = std::fs::write(file_handle.path(), &bytes);
                }
            }).detach();
        })
        .child(div().text_xs().text_color(pal.text_primary).child(format!("💾 {}", state.t("umii.export_bumii"))))
}

/// 渲染导出 YAML 文本文件按钮
pub fn render_export_yaml_button(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tid = tab_id.to_string();
    let model_cb = model.clone();
    div()
        .id("btn_export_yaml")
        .flex()
        .items_center()
        .gap_1()
        .px_2p5()
        .py_1()
        .rounded_md()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let tid = tid.clone();
            let model_cb = model_cb.clone();
            cx.spawn(async move |cx| {
                let opt_yaml: Option<String> = cx.update(|cx| {
                    model_cb.read(cx).open_tabs.iter()
                        .find(|t| t.id == tid)
                        .and_then(|t| t.umii_state.as_ref())
                        .map(|u| u.data.to_yaml_text())
                });
                let Some(text) = opt_yaml else { return; };
                if let Some(file_handle) = rfd::AsyncFileDialog::new()
                    .set_file_name("custom_npc.umii.yml")
                    .add_filter("BotW UMii YAML", &["yml", "yaml"])
                    .save_file()
                    .await
                {
                    let _ = std::fs::write(file_handle.path(), text.as_bytes());
                }
            }).detach();
        })
        .child(div().text_xs().text_color(pal.text_primary).child(format!("📄 {}", state.t("umii.export_yaml"))))
}
