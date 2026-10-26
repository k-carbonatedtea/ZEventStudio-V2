use gpui::*;
use super::prop_list::render_prop_list;
use crate::model::NodeData;
use crate::ui::state::AppState;

/// 渲染属性检查器中的选中节点详细信息面板
pub fn render_node_detail(
    state: &AppState,
    node: &NodeData,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_del = model.clone();
    let (h, s, l) = node.kind.header_color();

    div()
        .flex()
        .flex_col()
        .gap_3p5()
        // 1. 节点基本卡片 (图标、标题、ID、Actor/Scope)
        .child(
            div()
                .p_3()
                .rounded_lg()
                .bg(hsla(h, s, l, 0.15))
                .border_1()
                .border_color(hsla(h, s, l, 0.45))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(div().text_size(px(14.0)).child(node.kind.icon()))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(node.title.clone()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .child(format!("{}: {}", state.t("inspector.node_id"), node.id)),
                )
                .children(node.subtitle.as_ref().map(|sub| {
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(pal.secondary_accent)
                        .child(format!("Actor / Scope: {}", sub))
                })),
        )
        // 2. 画布坐标 (Canvas Position)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .child(state.t("inspector.position")),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(pal.card_surface) // 浅层蓝灰 Card / Hover
                                .border_1()
                                .border_color(pal.card_border)
                                .text_size(px(11.0))
                                .text_color(pal.text_primary)
                                .child(format!("X: {:.1}", node.pos.0)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(pal.card_surface) // 浅层蓝灰 Card / Hover
                                .border_1()
                                .border_color(pal.card_border)
                                .text_size(px(11.0))
                                .text_color(pal.text_primary)
                                .child(format!("Y: {:.1}", node.pos.1)),
                        ),
                ),
        )
        // 3. 端口引脚摘要 (Port Sockets Summary)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .child(format!(
                            "{} ({}: {} / {}: {})",
                            state.t("inspector.ports_summary"),
                            state.t("inspector.port_in"),
                            node.inputs.len(),
                            state.t("inspector.port_out"),
                            node.outputs.len()
                        )),

                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(node.inputs.iter().map(|p| {
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.primary_accent)
                                .text_size(px(10.5))
                                .text_color(pal.primary_accent) // 遗迹青蓝 Primary Accent
                                .child(format!("{}: {}", state.t("inspector.port_in"), p.name))
                        }))
                        .children(node.outputs.iter().map(|p| {
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.secondary_accent)
                                .text_size(px(10.5))
                                .text_color(pal.secondary_accent) // 琥珀金 Secondary Accent
                                .child(format!("{}: {}", state.t("inspector.port_out"), p.name))
                        })),
                ),
        )
        // 3.5. 自动填充与规范重排参数按钮 (针对 Action / Query / Switch 节点)
        .children(
            if matches!(node.kind, crate::model::NodeKind::Action | crate::model::NodeKind::Query | crate::model::NodeKind::Switch) {
                let n_id = node.id.clone();
                let n_id_sort = node.id.clone();
                let model_autofill = model.clone();
                let model_sort = model.clone();
                let has_props = !node.properties.is_empty();
                Some(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .flex_1()
                                .py_1p5()
                                .rounded_lg()
                                .bg(hsla(215.0 / 360.0, 0.45, 0.32, 1.0))
                                .hover(|s| s.bg(hsla(215.0 / 360.0, 0.55, 0.40, 1.0)))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_autofill.update(cx, |st, cx| {
                                        st.autofill_node_parameters(&n_id);
                                        cx.notify();
                                    });
                                })
                                .child(state.t("inspector.autofill_btn")),
                        )
                        .children(has_props.then(|| {
                            div()
                                .px_2p5()
                                .py_1p5()
                                .rounded_lg()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.card_border)
                                .hover(|s| s.bg(pal.card_hover))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_sort.update(cx, |st, cx| {
                                        st.reorder_node_parameters(&n_id_sort);
                                        cx.notify();
                                    });
                                })
                                .child(format!("↓↑ {}", state.t("modal.sort_params")))
                        })),
                )
            } else {
                None
            }
        )
        // 4. 对白内容预览 (若该节点配置了 MessageId / 对白)
        .children(render_inspector_dialogue(state, node))
        // 5. EVFL 参数属性列表
        .child(render_prop_list(state, &node.properties))
        // 6. 删除节点按钮
        .child(
            div()
                .mt_2()
                .py_2()
                .rounded_lg()
                .bg(rgb(0xEF4444))
                .hover(|s| s.opacity(0.9))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xFFFFFF))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_del.update(cx, |st, cx| {
                        st.delete_selected_node();
                        cx.notify();
                    });
                })
                .child(state.t("inspector.delete_node")),
        )
}

/// 渲染属性检查器中的对白多语言内容卡片
fn render_inspector_dialogue(state: &AppState, node: &NodeData) -> Option<AnyElement> {
    let (file_key, label) = crate::model::extract_node_message_target(node)?;
    let pal = state.theme();
    let dialogue = crate::model::resolve_node_dialogue(
        node,
        state.game_lang,
        state.project_path.as_deref(),
        &state.game_paths,
    );

    let (source_badge, source_bg, source_fg) = if let Some(ref item) = dialogue {
        match item.source {
            crate::model::DialogueSource::Project => (
                state.t("node.dialogue_source_project"),
                hsla(160.0 / 360.0, 0.40, 0.20, 0.85),
                hsla(160.0 / 360.0, 0.90, 0.70, 1.0),
            ),
            crate::model::DialogueSource::GameDump => (
                state.t("node.dialogue_source_dump"),
                hsla(38.0 / 360.0, 0.40, 0.20, 0.85),
                hsla(38.0 / 360.0, 0.95, 0.70, 1.0),
            ),
        }
    } else {
        (
            state.t("node.dialogue_source_project"),
            hsla(215.0 / 360.0, 0.20, 0.20, 0.50),
            hsla(215.0 / 360.0, 0.20, 0.60, 1.0),
        )
    };

    Some(
        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(pal.text_primary)
                            .child(state.t("inspector.dialogue_preview")),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(pal.card_surface)
                                    .text_size(px(10.0))
                                    .text_color(pal.primary_accent)
                                    .child(state.game_lang.code()),
                            )
                            .children(dialogue.is_some().then(|| {
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(source_bg)
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(source_fg)
                                    .child(source_badge)
                            })),
                    ),
            )
            .child(
                div()
                    .p_3()
                    .rounded_lg()
                    .bg(pal.card_surface)
                    .border_1()
                    .border_color(pal.card_border)
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .text_size(px(10.5))
                            .text_color(pal.secondary_accent)
                            .child(format!("{}: {}", file_key, label)),
                    )
                    .child(if let Some(item) = dialogue {
                        div()
                            .text_size(px(11.5))
                            .text_color(pal.text_primary)
                            .child(item.text)
                            .into_any_element()
                    } else {
                        div()
                            .text_size(px(11.0))
                            .text_color(pal.text_secondary)
                            .italic()
                            .child(state.t("node.dialogue_not_found"))
                            .into_any_element()
                    }),
            )
            .into_any_element(),
    )
}
