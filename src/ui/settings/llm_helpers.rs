use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::model::settings::LlmProvider;
use crate::ui::state::AppState;
use crate::ui::settings::general_helpers::render_choice_chip;

pub fn browse_ai_dump_folder(model: &Entity<AppState>, cx: &mut App) {
    let m = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .set_title("选择包含 .sbyml / .baiprog 的 AI 参数导出目录")
            .pick_folder()
            .await;

        if let Some(folder) = handle {
            let path_str = folder.path().to_string_lossy().to_string();
            cx.update(|cx| {
                m.update(cx, |st, cx| {
                    st.settings_draft.custom_ai_dump_dir = path_str;
                    cx.notify();
                });
            });
        }
    })
    .detach();
}

pub fn render_ai_dump_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.custom_ai_dump_dir.clone();
    let m = model.clone();

    crate::ui::settings::switch_pane::render_path_field(
        "游戏 AI 参数目录 (Game AI Directory)",
        "自定义覆盖查找字典: 用于优先搜索并解析 .sbyml / .baiprog 事件属性定义。若留空将回退在 Switch/Wii U 游戏目录下搜索",
        &path_val,
        "D:\\Game_Dump\\content\\Pack\\Bootup.pack (extracted)",
        crate::ui::state::ActiveSettingsField::AiDumpDir,
        state,
        model,
        move |cx| {
            browse_ai_dump_folder(&m, cx);
        },
    )
}

pub fn render_llm_enable_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
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

pub fn render_llm_provider_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
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
                .flex_wrap()
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
