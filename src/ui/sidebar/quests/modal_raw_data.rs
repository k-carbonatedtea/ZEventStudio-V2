use gpui::*;
use crate::ui::sidebar::quests::modal_raw_format::generate_raw_data_text;
use crate::ui::sidebar::quests::modal_state::{QuestRawDataModalState, RawDataFormat};
use crate::ui::state::AppState;

/// 渲染任务原始数据与拓扑查看模态框
pub fn render_quest_raw_data_modal(
    modal: &QuestRawDataModalState,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_close = model.clone();
    let model_backdrop = model.clone();
    let model_edit = model.clone();
    let model_copy = model.clone();

    let quest = &modal.quest;
    let quest_clone = quest.clone();
    let current_format = modal.active_format;

    let content_text = generate_raw_data_text(modal);
    let copy_text = content_text.clone();

    let (badge_bg, badge_fg) = match quest.quest_type {
        crate::model::quest::QuestType::Main => (pal.secondary_accent.opacity(0.18), pal.secondary_accent),
        crate::model::quest::QuestType::Sub => (pal.primary_accent.opacity(0.18), pal.primary_accent),
        crate::model::quest::QuestType::Mini => (pal.primary_accent_hover.opacity(0.18), pal.primary_accent_hover),
    };

    div()
        .id("quest_raw_data_modal_overlay")
        .absolute()
        .inset_0()
        .occlude()
        .bg(rgba(0x00000088))
        .flex()
        .items_center()
        .justify_center()
        .on_scroll(|_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            model_backdrop.update(cx, |st, cx| {
                st.close_quest_raw_data();
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_click(|_, _, cx| {
            cx.stop_propagation();
        })
        .child(
            div()
                .id("quest_raw_data_modal_window")
                .w(px(720.0))
                .max_h(px(640.0))
                .rounded_lg()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.primary_accent.opacity(0.35))
                .shadow_lg()
                .flex()
                .flex_col()
                .overflow_hidden()
                .on_scroll(|_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_up(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_up(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_click(|_, _, cx| {
                    cx.stop_propagation();
                })
                // 1. 弹窗头部：图标、任务名、分类 Badge 与操作快捷按钮
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .border_b_1()
                        .border_color(pal.panel_border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().text_size(px(16.0)).child("🔍"))
                                .child(
                                    div()
                                        .text_size(px(13.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_primary)
                                        .child(format!("{}: {}", state.t("quests.raw_data_title"), quest.name)),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(badge_bg)
                                        .text_color(badge_fg)
                                        .child(crate::i18n::t(quest.quest_type.label_key(), state.lang)),
                                )
                                .children(quest.display_title.as_ref().map(|title| {
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(pal.text_secondary)
                                        .child(format!("({})", title))
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                // 一键复制按钮
                                .child(
                                    div()
                                        .id("btn_copy_raw_data")
                                        .h(px(26.0))
                                        .px_2p5()
                                        .rounded_sm()
                                        .border_1()
                                        .border_color(pal.panel_border)
                                        .bg(pal.card_hover.opacity(0.5))
                                        .text_size(px(11.0))
                                        .text_color(pal.text_secondary)
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            cx.write_to_clipboard(ClipboardItem::new_string(copy_text.clone()));
                                            model_copy.update(cx, |st, cx| {
                                                st.toast = Some(crate::ui::components::Toast::success(st.t("quests.raw_data_copied")));
                                                cx.notify();
                                            });
                                        })
                                        .child("📋")
                                        .child(state.t("quests.btn_copy")),
                                )
                                // 编辑此任务按钮
                                .child(
                                    div()
                                        .id("btn_edit_from_raw_modal")
                                        .h(px(26.0))
                                        .px_2p5()
                                        .rounded_sm()
                                        .bg(pal.primary_accent.opacity(0.18))
                                        .border_1()
                                        .border_color(pal.primary_accent.opacity(0.4))
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(pal.primary_accent)
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .hover(|s| s.bg(pal.primary_accent.opacity(0.28)))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            let q = quest_clone.clone();
                                            model_edit.update(cx, |st, cx| {
                                                st.close_quest_raw_data();
                                                st.open_quest_editor(&q);
                                                cx.notify();
                                            });
                                        })
                                        .child("✏️")
                                        .child(state.t("quests.btn_edit_quest")),
                                )
                                // 关闭按钮
                                .child(
                                    div()
                                        .id("btn_close_raw_modal")
                                        .size(px(26.0))
                                        .rounded_sm()
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(px(13.0))
                                        .text_color(pal.text_secondary)
                                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            model_close.update(cx, |st, cx| {
                                                st.close_quest_raw_data();
                                                cx.notify();
                                            });
                                        })
                                        .child("✕"),
                                ),
                        ),
                )
                // 2. 格式切换 Tabs
                .child(render_raw_format_tabs(current_format, state, model))
                // 3. 代码展示容器 (等宽字体、滚动区、深色背景)
                .child(
                    div()
                        .id("quest_raw_data_content_box")
                        .flex_1()
                        .max_h(px(460.0))
                        .overflow_y_scroll()
                        .p_3()
                        .bg(rgb(0x111318))
                        .border_t_1()
                        .border_b_1()
                        .border_color(pal.panel_border)
                        .child(
                            div()
                                .w_full()
                                .font_family("Consolas, 'Courier New', monospace")
                                .text_size(px(11.5))
                                .line_height(px(18.0))
                                .text_color(rgb(0xdfdfdf))
                                .whitespace_nowrap()
                                .children(content_text.lines().enumerate().map(|(idx, line)| {
                                    div()
                                        .flex()
                                        .items_center()
                                        .hover(|s| s.bg(rgb(0x1a1e28)))
                                        .child(
                                            div()
                                                .w(px(34.0))
                                                .text_color(rgb(0x606575))
                                                .text_size(px(10.5))
                                                .child(format!("{:>3}", idx + 1)),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_color(highlight_code_line(line))
                                                .child(line.to_string()),
                                        )
                                })),
                        ),
                )
                // 4. 底部状态栏说明
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_2()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(format!("阶段数量: {} | 标点总数: {}", quest.steps.len(), quest.total_indicators_count()))
                        .child("按 Esc 键关闭"),
                ),
        )
}

/// 渲染格式选项卡切换栏
fn render_raw_format_tabs(
    current: RawDataFormat,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let tabs = [
        (RawDataFormat::Json, state.t("quests.raw_tab_json")),
        (RawDataFormat::BymlYaml, state.t("quests.raw_tab_byml")),
        (RawDataFormat::Msbt, state.t("quests.raw_tab_msbt")),
        (RawDataFormat::FlagsHash, state.t("quests.raw_tab_flags")),
    ];

    div()
        .flex()
        .items_center()
        .px_3()
        .py_1p5()
        .gap_1p5()
        .bg(pal.panel_surface)
        .children(tabs.into_iter().map(|(fmt, label)| {
            let is_active = current == fmt;
            let model_fmt = model.clone();

            div()
                .id(SharedString::from(format!("tab_raw_fmt_{:?}", fmt)))
                .px_2p5()
                .py_1()
                .rounded_sm()
                .text_size(px(11.0))
                .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                .cursor_pointer()
                .bg(if is_active { pal.primary_accent.opacity(0.18) } else { pal.card_hover.opacity(0.4) })
                .text_color(if is_active { pal.primary_accent } else { pal.text_secondary })
                .border_1()
                .border_color(if is_active { pal.primary_accent.opacity(0.5) } else { rgba(0x00000000) })
                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_fmt.update(cx, |st, cx| {
                        if let Some(ref mut m) = st.quest_raw_data_modal {
                            m.active_format = fmt;
                            cx.notify();
                        }
                    });
                })
                .child(label)
        }))
}

/// 简单高亮代码行颜色
fn highlight_code_line(line: &str) -> Rgba {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        rgb(0x6a9955) // 注释绿
    } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
        rgb(0x4fc1ff) // 节标题青
    } else if trimmed.starts_with('"') && trimmed.contains("\":") {
        rgb(0x9cdcfe) // JSON Key 浅蓝
    } else if trimmed.contains(':') && !trimmed.starts_with("http") {
        rgb(0xdcdcaa) // YAML 键值黄
    } else if trimmed.starts_with("0x") {
        rgb(0xb5cea8) // 十六进制数绿
    } else {
        rgb(0xcccccc)
    }
}
