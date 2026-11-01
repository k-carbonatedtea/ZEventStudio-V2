//! UMii 编辑器顶部操作工具栏 (包含平台切换、随机生成、恢复默认、Mii导入、格式导出与 3D 弹出预览)

use gpui::*;
use crate::model::umii::{collect_model_search_dirs, ModelPlatform};
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;

pub fn render_umii_toolbar(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
) -> impl IntoElement {
    let Some(active_id) = &state.active_tab_id else {
        return div().id("umii_toolbar_empty");
    };
    let Some(tab) = state.open_tabs.iter().find(|t| &t.id == active_id) else {
        return div().id("umii_toolbar_empty");
    };
    let Some(umii_state) = &tab.umii_state else {
        return div().id("umii_toolbar_empty");
    };

    let platform = umii_state.target_platform;
    let is_modified = umii_state.is_modified();
    let file_title = tab.title.clone();
    let has_model = umii_state.synthesized_model.is_some();
    let tab_id = tab.id.clone();

    let model_plat = model.clone();
    let id_plat = tab_id.clone();

    let model_rand = model.clone();
    let id_rand = tab_id.clone();

    let model_reset = model.clone();
    let id_reset = tab_id.clone();

        let model_resynth = model.clone();
    let id_resynth = tab_id.clone();

    let model_popout = model.clone();
    let id_popout = tab_id.clone();

    div()
        .id("umii_toolbar")
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .py_2()
        .bg(pal.card_surface)
        .border_b_1()
        .border_color(pal.card_border)
        // 左侧：文件标识与修改状态徽章
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .child(format!("👤 {}", file_title)),
                )
                .children(is_modified.then(|| {
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(rgb(0x854D0E))
                        .text_xs()
                        .text_color(rgb(0xFEF08A))
                        .child(state.t("umii.modified"))
                })),
        )
        // 右侧操作按钮组
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                // 1. 目标平台切换按钮 (Switch / Wii U)
                .child(
                    div()
                        .id("btn_toggle_platform")
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(if platform == ModelPlatform::Switch { rgb(0x1E3A8A) } else { rgb(0x065F46) })
                        .border_1()
                        .border_color(if platform == ModelPlatform::Switch { rgb(0x3B82F6) } else { rgb(0x10B981) })
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let next_plat = if platform == ModelPlatform::Switch {
                                ModelPlatform::WiiU
                            } else {
                                ModelPlatform::Switch
                            };
                            let tid = id_plat.clone();
                            model_plat.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.target_platform = next_plat;
                                        let dirs = collect_model_search_dirs(next_plat, st.project_path.as_deref());
                                        let _ = u.synthesize_model(&dirs);
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .child(match platform {
                                    ModelPlatform::Switch => format!("🎮 {}", state.t("umii.platform_switch")),
                                    ModelPlatform::WiiU => format!("💿 {}", state.t("umii.platform_wiiu")),
                                }),
                        ),
                )
                // 2. 🎲 随机形象生成按钮 (MyAvatarEditor 经典体验)
                .child(
                    div()
                        .id("btn_randomize_avatar")
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(rgb(0x7C3AED))
                        .border_1()
                        .border_color(rgb(0xA78BFA))
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let tid = id_rand.clone();
                            model_rand.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.randomize();
                                        t.is_modified = true;
                                        let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                        let _ = u.synthesize_model(&dirs);
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .child(format!("🎲 {}", state.t("umii.randomize"))),
                        ),
                )
                // 3. 🔄 恢复默认按钮
                .children(is_modified.then(|| {
                    div()
                        .id("btn_reset_avatar")
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(rgb(0xEF4444))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0xEF4444)).text_color(rgb(0xFFFFFF)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let tid = id_reset.clone();
                            model_reset.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.reset_to_original();
                                        t.is_modified = false;
                                        let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                        let _ = u.synthesize_model(&dirs);
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xEF4444))
                                .child(format!("↺ {}", state.t("umii.reset"))),
                        )
                }))
                // 4. 重新合成 3D 模型
                .child(
                    div()
                        .id("btn_resynthesize_umii")
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
                            let tid = id_resynth.clone();
                            model_resynth.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                        let _ = u.synthesize_model(&dirs);
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .child(div().text_xs().text_color(pal.text_primary).child(format!("🔄 {}", state.t("umii.resynthesize")))),
                )
                // 5. 导入外部 Mii 文件
                .child(crate::ui::views::umii_editor::toolbar_io::render_import_button(state, model, pal, &tab_id))
                // 6. 导出 BUMII
                .child(crate::ui::views::umii_editor::toolbar_io::render_export_bumii_button(state, model, pal, &tab_id))
                // 7. 导出 YAML
                .child(crate::ui::views::umii_editor::toolbar_io::render_export_yaml_button(state, model, pal, &tab_id))
                // 8. 独立视口弹出 (仅在已有合成模型时高亮)
                .children(has_model.then(|| {
                    div()
                        .id("btn_popout_3d")
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(pal.primary_accent)
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let tid = id_popout.clone();
                            model_popout.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter().find(|t| t.id == tid) {
                                    if let Some(ref u) = t.umii_state {
                                        if let Some(ref m_doc) = u.synthesized_model {
                                            let target = crate::model::model_preview::actor_resolver::ActorModelTarget {
                                                folder: format!("UMii_{}", u.data.name),
                                                unit_names: vec![],
                                                actor_name: u.data.name.clone(),
                                            };
                                            let _ = crate::ui::views::model_preview::open_standalone_model_preview_window(
                                                target,
                                                m_doc.clone(),
                                                st.lang,
                                                cx,
                                            );
                                        }
                                    }
                                }
                            });
                        })
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x0F1117))
                                .child(format!("🧊 {}", state.t("umii.open_standalone_3d"))),
                        )
                })),
        )
}
