use gpui::*;
use crate::model::FileNode;
use crate::ui::state::{AppState, FileFilterType};
use std::sync::Arc;

pub mod types;
pub mod search_sort_bar;
pub mod filter;
pub mod row;

#[allow(unused_imports)]
pub use types::*;
pub use search_sort_bar::*;
pub use filter::*;
pub(crate) use row::render_flat_file_row;

pub fn render_file_tree(
    nodes: &[FileNode],
    state: &AppState,
    model: &Entity<AppState>,
    _level: usize,
) -> impl IntoElement {
    let filter_type = state.file_tree_filter_type;
    let is_filtering = filter_type != FileFilterType::All;
    let selected_path = state.selected_file_path.as_deref().unwrap_or("");
    let pal = state.theme();

    let mut matching_paths = std::collections::HashSet::new();
    if is_filtering {
        collect_matching_paths(
            nodes,
            filter_type,
            state,
            &mut matching_paths,
        );
    }

    let mut flat_entries = Vec::new();
    flatten_visible_nodes(
        nodes,
        &state.expanded_tree_paths,
        is_filtering,
        &matching_paths,
        selected_path,
        state,
        0,
        &mut flat_entries,
    );

    let count = flat_entries.len();
    if count == 0 {
        let model_reset = model.clone();
        return div()
            .p_4()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(pal.text_secondary)
                    .child(if !is_filtering {
                        state.t("welcome.no_folder_hint")
                    } else {
                        state.t("explorer.no_match")
                    }),
            )
            .children(is_filtering.then(|| {
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(pal.card_surface)
                    .hover(|s| s.bg(pal.card_hover))
                    .cursor_pointer()
                    .text_size(px(11.0))
                    .text_color(pal.primary_accent)
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        model_reset.update(cx, |st, cx| {
                            st.file_search_query.clear();
                            st.file_tree_filter_type = FileFilterType::All;
                            cx.notify();
                        });
                    })
                    .child(state.t("explorer.reset_filter"))
            }))
            .into_any_element();
    }

    let flat_entries = Arc::new(flat_entries);
    let model_clone = model.clone();

    uniform_list("file_tree_virtual_list", count, move |range, _window, _cx| {
        let entries = flat_entries.clone();
        let m = model_clone.clone();
        range
            .map(|ix| {
                if let Some(item) = entries.get(ix) {
                    render_flat_file_row(item, &m, &pal).into_any_element()
                } else {
                    div().into_any_element()
                }
            })
            .collect()
    })
    .track_scroll(&state.file_tree_scroll_handle)
    .size_full()
    .into_any_element()
}
