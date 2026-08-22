use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::welcome::quick_tab::{pick_and_load_file, pick_and_load_mod_folder};

/// 渲染 VS Code 风格的左侧「资源管理器」侧边栏面板
pub fn render_welcome_sidebar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_folder = model.clone();
    let model_file = model.clone();
    let model_new = model.clone();

    div()
        .w(px(260.0))
        .h_full()
        .bg(hsla(220.0 / 360.0, 0.35, 0.08, 1.0))
        .border_r_1()
        .border_color(hsla(220.0 / 360.0, 0.25, 0.16, 1.0))
        .flex()
        .flex_col()
        // 顶部标题栏
        .child(
            div()
                .h(px(36.0))
                .px_4()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(hsla(220.0 / 360.0, 0.20, 0.14, 1.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(215.0 / 360.0, 0.20, 0.80, 1.0))
                        .child(state.t("welcome.explorer")),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(hsla(215.0 / 360.0, 0.20, 0.60, 1.0))
                        .child("•••"),
                ),
        )
        // 核心折叠区：无打开的文件夹
        .child(
            div()
                .id("welcome_sidebar_scroll")
                .flex_1()
                .overflow_y_scroll()
                .flex()
                .flex_col()

                // 折叠头
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .bg(hsla(220.0 / 360.0, 0.25, 0.11, 1.0))
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(0.0, 0.0, 1.0, 0.90))
                        .child("⌄")
                        .child(state.t("welcome.no_folder")),
                )
                // 折叠主体引导按钮
                .child(
                    div()
                        .p_4()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.65, 1.0))
                                .child(state.t("welcome.no_folder_hint")),
                        )
                        // 打开文件夹大按钮
                        .child(
                            div()
                                .w_full()
                                .py_2()
                                .px_3()
                                .rounded_md()
                                .bg(hsla(215.0 / 360.0, 0.60, 0.35, 1.0))
                                .hover(|s| s.bg(hsla(215.0 / 360.0, 0.70, 0.42, 1.0)))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    pick_and_load_mod_folder(&model_folder, cx);
                                })
                                .child("🗂️")
                                .child(state.t("welcome.open_folder_btn")),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(hsla(215.0 / 360.0, 0.20, 0.50, 1.0))
                                .child(state.t("welcome.open_folder_desc")),
                        )
                        // 打开单个文件次要按钮
                        .child(
                            div()
                                .w_full()
                                .py_1p5()
                                .px_3()
                                .rounded_md()
                                .bg(hsla(220.0 / 360.0, 0.35, 0.15, 1.0))
                                .border_1()
                                .border_color(hsla(220.0 / 360.0, 0.25, 0.22, 1.0))
                                .hover(|s| s.bg(hsla(215.0 / 360.0, 0.40, 0.20, 1.0)))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_size(px(11.5))
                                .text_color(hsla(215.0 / 360.0, 0.15, 0.85, 1.0))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    pick_and_load_file(&model_file, cx);
                                })
                                .child("📁")
                                .child(state.t("welcome.open_file_btn")),
                        )
                        // 新建空白事件流按钮
                        .child(
                            div()
                                .w_full()
                                .py_1p5()
                                .px_3()
                                .rounded_md()
                                .bg(hsla(220.0 / 360.0, 0.35, 0.15, 1.0))
                                .border_1()
                                .border_color(hsla(220.0 / 360.0, 0.25, 0.22, 1.0))
                                .hover(|s| s.bg(hsla(145.0 / 360.0, 0.40, 0.20, 1.0)))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_size(px(11.5))
                                .text_color(hsla(145.0 / 360.0, 0.50, 0.85, 1.0))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_new.update(cx, |st, cx| {
                                        st.new_blank_flow();
                                        cx.notify();
                                    });
                                })
                                .child("📄")
                                .child(state.t("welcome.create_flow_btn")),
                        ),
                ),
        )
}

