use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::model::settings::LlmProvider;
use crate::ui::state::AppState;
use crate::ui::settings::general_pane::render_choice_chip;

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
        // 1. LLM 启用开关
        .child(render_llm_enable_card(state, model))
        // 2. 服务提供商选择
        .child(render_llm_provider_card(state, model))
        // 3. 模型参数输入
        .child(render_llm_params_card(state, model))
}

fn render_llm_enable_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_enabled = state.settings_draft.llm.enable_auto_fill;
    let m = model.clone();

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("✨"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.llm_enable")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.llm_enable_desc")),
                        ),
                ),
        )
        .child(
            div()
                .w(px(36.0))
                .h(px(20.0))
                .rounded_full()
                .bg(if is_enabled {
                    pal.primary_accent
                } else {
                    pal.panel_surface
                })
                .border_1()
                .border_color(if is_enabled {
                    pal.primary_accent
                } else {
                    pal.panel_border
                })
                .flex()
                .items_center()
                .px_0p5()
                .cursor_pointer()
                .justify_start()
                .when(is_enabled, |s| s.justify_end())
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    m.update(cx, |st, cx| {
                        st.settings_draft.llm.enable_auto_fill = !is_enabled;
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .w(px(14.0))
                        .h(px(14.0))
                        .rounded_full()
                        .bg(if is_enabled {
                            rgb(0x0F1117)
                        } else {
                            pal.text_secondary
                        })
                        .shadow_sm(),
                ),
        )
}

fn render_llm_provider_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_provider = state.settings_draft.llm.provider;

    let m_ds = model.clone();
    let m_oai = model.clone();
    let m_ollama = model.clone();

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("🧠"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.llm_provider")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.llm_provider_desc")),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(render_choice_chip(
                    "DeepSeek",
                    current_provider == LlmProvider::DeepSeek,
                    &pal,
                    move |cx| {
                        m_ds.update(cx, |st, cx| {
                            st.settings_draft.llm.provider = LlmProvider::DeepSeek;
                            st.settings_draft.llm.base_url = "https://api.deepseek.com".to_string();
                            st.settings_draft.llm.model_name = "deepseek-coder".to_string();
                            cx.notify();
                        });
                    },
                ))
                .child(render_choice_chip(
                    "OpenAI",
                    current_provider == LlmProvider::OpenAI,
                    &pal,
                    move |cx| {
                        m_oai.update(cx, |st, cx| {
                            st.settings_draft.llm.provider = LlmProvider::OpenAI;
                            st.settings_draft.llm.base_url = "https://api.openai.com/v1".to_string();
                            st.settings_draft.llm.model_name = "gpt-4o-mini".to_string();
                            cx.notify();
                        });
                    },
                ))
                .child(render_choice_chip(
                    "Ollama (Local)",
                    current_provider == LlmProvider::Ollama,
                    &pal,
                    move |cx| {
                        m_ollama.update(cx, |st, cx| {
                            st.settings_draft.llm.provider = LlmProvider::Ollama;
                            st.settings_draft.llm.base_url = "http://localhost:11434/v1".to_string();
                            st.settings_draft.llm.model_name = "deepseek-coder-v2".to_string();
                            cx.notify();
                        });
                    },
                )),
        )
}

fn render_llm_params_card(state: &AppState, _model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let llm = &state.settings_draft.llm;

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
                .child(
                    div()
                        .mt_1()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .child(llm.base_url.clone()),
                ),
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
                .child(
                    div()
                        .mt_1()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .child(llm.model_name.clone()),
                ),
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
                .child(
                    div()
                        .mt_1()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(12.0))
                        .text_color(pal.text_primary)
                        .child(if llm.api_key.is_empty() {
                            "****************"
                        } else {
                            "****************"
                        }),
                ),
        )
}
