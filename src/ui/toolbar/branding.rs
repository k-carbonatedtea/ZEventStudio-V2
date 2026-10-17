use gpui::*;
use crate::ui::state::AppState;

/// 渲染顶部工具栏左侧 Branding 区域 (Logo、应用标题、当前图谱信息与统计徽章)
pub fn render_branding(state: &AppState) -> impl IntoElement {
    let pal = state.theme();
    let node_count = state.graph.nodes.len();
    let edge_count = state.graph.edges.len();
    let node_label = state.t("toolbar.nodes_label");
    let edge_label = state.t("toolbar.edges_label");

    // 活跃文档摘要徽章 (仅在非欢迎页且存在打开文档标签时显示)
    let active_tab = state
        .active_tab_id
        .as_deref()
        .and_then(|id| state.open_tabs.iter().find(|t| t.id == id));

    let active_badge = if state.current_tab != crate::ui::state::AppWorkspaceTab::Welcome {
        active_tab.map(|tab| match tab.tab_kind {
            crate::model::DocumentTabKind::Flowchart => {
                format!(
                    "{} ({} {} / {} {})",
                    tab.title, node_count, node_label, edge_count, edge_label
                )
            }
            crate::model::DocumentTabKind::CodeEditor => {
                if let Some(ref code) = tab.code_state {
                    format!(
                        "{} [{}] ({} {}, {} {})",
                        tab.title,
                        code.format.display_name(),
                        code.content.lines().count(),
                        state.t("code_editor.lines"),
                        code.content.len(),
                        state.t("code_editor.chars")
                    )
                } else {
                    tab.title.clone()
                }
            }
            crate::model::DocumentTabKind::DiffViewer => {
                if let Some(ref diff_state) = tab.diff_state {
                    let d = diff_state.current_diff();
                    format!("{} (+{} / -{})", tab.title, d.additions, d.deletions)
                } else {
                    tab.title.clone()
                }
            }
        })
    } else {
        None
    };

    div()
        .flex()
        .items_center()
        .gap_3()
        // Logo & Title
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_size(px(18.0))
                        .text_color(pal.secondary_accent)
                        .child("⚡"),
                )
                .child(
                    div()
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_size(px(15.0))
                        .text_color(pal.text_primary)
                        .child("NodeFlow Studio"),
                ),
        )
        // Studio Tag
        .child(
            div()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
                .child("EVFL Studio"),
        )
        // Active Document Summary Badge
        .children(active_badge.map(|badge_text| {
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.primary_accent)
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.primary_accent)
                .child(badge_text)
        }))
}
