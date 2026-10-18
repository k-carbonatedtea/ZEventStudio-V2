use gpui::*;
use crate::ui::state::AppState;
/// 渲染未初始化 Git 仓库状态或未打开文件夹视图
pub fn render_uninitialized_repo_view(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_init = model.clone();
    let model_open_folder = model.clone();
    let has_project_folder = state.get_git_working_dir().is_some();

    if has_project_folder {
        div()
            .p_5()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .text_center()
            .child(
                div()
                    .text_size(px(32.0))
                    .child("🌿"),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(pal.text_primary)
                    .child(state.t("source_control.not_a_repo")),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(pal.text_secondary)
                    .line_height(relative(1.4))
                    .child(state.t("source_control.not_a_repo_hint")),
            )
            .child(
                div()
                    .mt_2()
                    .px_4()
                    .py_1p5()
                    .rounded_md()
                    .bg(pal.primary_accent)
                    .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                    .text_size(px(11.5))
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.9))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        model_init.update(cx, |st, cx| {
                            st.git_init_repository();
                            cx.notify();
                        });
                    })
                    .child(state.t("source_control.init_repo")),
            )
    } else {
        div()
            .p_5()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .text_center()
            .child(
                div()
                    .text_size(px(32.0))
                    .child("📁"),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(pal.text_primary)
                    .child(state.t("source_control.no_folder_open")),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(pal.text_secondary)
                    .line_height(relative(1.4))
                    .child(state.t("source_control.no_folder_open_hint")),
            )
            .child(
                div()
                    .mt_2()
                    .px_4()
                    .py_1p5()
                    .rounded_md()
                    .bg(pal.primary_accent)
                    .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                    .text_size(px(11.5))
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.9))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        crate::ui::views::welcome::quick_tab::pick_and_load_mod_folder(&model_open_folder, cx);
                    })
                    .child(state.t("explorer.open_mod")),
            )
    }
}
