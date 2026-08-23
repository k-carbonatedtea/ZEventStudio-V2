use gpui::*;
use crate::ui::state::AppState;
use super::llm_helpers::{render_ai_dump_dir_card, render_llm_enable_card, render_llm_provider_card};

/// 渲染大语言模型 (LLM) 配置面板
pub fn render_llm_pane(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    div()
        .id("settings_llm_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 1. 本地 AI 定义文件夹 (Game AI Parameter Dump Dir)
        .child(render_ai_dump_dir_card(state, model))
        // 2. LLM 启用开关
        .child(render_llm_enable_card(state, model))
        // 3. 服务提供商选择
        .child(render_llm_provider_card(state, model))
        // 4. 模型参数输入
        .child(render_llm_params_card(state, model))
}

fn render_llm_params_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let llm = &state.settings_draft.llm;
    let m = model.clone();

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .child(state.t("settings.llm_base_url")),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("settings.llm_base_url_desc")),
                )
                .child({
                    let is_active = state.active_settings_field == crate::ui::state::ActiveSettingsField::LlmBaseUrl;
                    let m_clone = m.clone();
                    div()
                        .mt_1()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m_clone.update(cx, |st, cx| {
                                st.active_settings_field = crate::ui::state::ActiveSettingsField::LlmBaseUrl;
                                cx.notify();
                            });
                        })
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .flex()
                        .items_center()
                        .child(llm.base_url.clone())
                        .children(if is_active {
                            Some(crate::ui::components::text_input::render_blinking_cursor(pal.primary_accent.into()))
                        } else {
                            None
                        })
                }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .child(state.t("settings.llm_model_name")),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("settings.llm_model_name_desc")),
                )
                .child({
                    let is_active = state.active_settings_field == crate::ui::state::ActiveSettingsField::LlmModelName;
                    let m_clone = m.clone();
                    div()
                        .mt_1()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m_clone.update(cx, |st, cx| {
                                st.active_settings_field = crate::ui::state::ActiveSettingsField::LlmModelName;
                                cx.notify();
                            });
                        })
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .flex()
                        .items_center()
                        .child(llm.model_name.clone())
                        .children(if is_active {
                            Some(crate::ui::components::text_input::render_blinking_cursor(pal.primary_accent.into()))
                        } else {
                            None
                        })
                }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .child(state.t("settings.llm_api_key")),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("settings.llm_api_key_desc")),
                )
                .child({
                    let is_active = state.active_settings_field == crate::ui::state::ActiveSettingsField::LlmApiKey;
                    let m_clone = m.clone();
                    div()
                        .mt_1()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m_clone.update(cx, |st, cx| {
                                st.active_settings_field = crate::ui::state::ActiveSettingsField::LlmApiKey;
                                cx.notify();
                            });
                        })
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .flex()
                        .items_center()
                        .child(if llm.api_key.is_empty() && !is_active {
                            "****************".to_string()
                        } else {
                            llm.api_key.clone()
                        })
                        .children(if is_active {
                            Some(crate::ui::components::text_input::render_blinking_cursor(pal.primary_accent.into()))
                        } else {
                            None
                        })
                }),
        )
}
