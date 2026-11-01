//! UMii 交互式 3D 角色形象编辑器主视图 (集成 9 大分类控制台与 3D 实时视口)

pub mod accessories_panel;
pub mod basic_panel;
pub mod beard_panel;
pub mod components;
pub mod eyebrows_panel;
pub mod eyes_panel;
pub mod face_panel;
pub mod hair_panel;
pub mod mouth_panel;
pub mod nose_panel;
pub mod toolbar;
pub mod toolbar_io;
pub mod viewport;

use gpui::*;
use crate::model::umii::UmiiEditorSection;
use crate::ui::state::AppState;
use accessories_panel::render_accessories_panel;
use basic_panel::render_basic_panel;
use beard_panel::render_beard_panel;
use eyebrows_panel::render_eyebrows_panel;
use eyes_panel::render_eyes_panel;
use face_panel::render_face_panel;
use hair_panel::render_hair_panel;
use mouth_panel::render_mouth_panel;
use nose_panel::render_nose_panel;
use toolbar::render_umii_toolbar;
use viewport::render_umii_viewport;

/// 渲染全屏 UMii 形象编辑器主视图
pub fn render_umii_editor_view(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let Some(active_id) = &state.active_tab_id else {
        return div().id("umii_view_empty").child("未选择活跃文档");
    };
    let Some(tab) = state.open_tabs.iter().find(|t| &t.id == active_id) else {
        return div().id("umii_view_empty").child("文档已关闭");
    };
    let Some(umii_state) = &tab.umii_state else {
        return div().id("umii_view_empty").child("无有效的 UMii 数据");
    };

    let active_sec = umii_state.active_section;
    let tab_id = tab.id.clone();
    let model_doc = umii_state.synthesized_model.as_ref();

    div()
        .id("umii_editor_view")
        .size_full()
        .flex()
        .flex_col()
        .bg(pal.app_bg)
        .overflow_hidden()
        // 1. 顶部操作栏 (平台切换、随机形象、恢复默认、导入导出)
        .child(render_umii_toolbar(state, model, &pal))
        // 2. 主体工作区 (左侧分类导航 + 中间参数配置面板 + 右侧 3D 实时渲染视口)
        .child(
            div()
                .flex_1()
                .size_full()
                .flex()
                .overflow_hidden()
                // 左侧与中间：分类导航 + 参数控制面板
                .child(
                    div()
                        .w(px(420.0))
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .bg(pal.panel_surface)
                        .border_r_1()
                        .border_color(pal.card_border)
                        .overflow_hidden()
                        // 顶部类别导航胶囊条
                        .child(render_section_navigation(active_sec, &tab_id, model, &pal, state))
                        // 活跃分类面板内容区 (可纵向滚动)
                        .child(
                            div()
                                .id("umii_panel_scroll")
                                .flex_1()
                                .size_full()
                                .overflow_y_scroll()
                                .child(match active_sec {
                                    UmiiEditorSection::Basic => render_basic_panel(state, model, &pal, &tab_id).into_any_element(),
                                    UmiiEditorSection::Face => render_face_panel(state, model, &pal, &tab_id).into_any_element(),
                                    UmiiEditorSection::Hair => render_hair_panel(state, model, &pal, &tab_id).into_any_element(),
                                    UmiiEditorSection::Eyebrows => render_eyebrows_panel(state, model, &pal, &tab_id).into_any_element(),
                                    UmiiEditorSection::Eyes => render_eyes_panel(state, model, &pal, &tab_id).into_any_element(),
                                    UmiiEditorSection::Nose => render_nose_panel(state, model, &pal, &tab_id).into_any_element(),
                                    UmiiEditorSection::Mouth => render_mouth_panel(state, model, &pal, &tab_id).into_any_element(),
                                    UmiiEditorSection::Beard => render_beard_panel(state, model, &pal, &tab_id).into_any_element(),
                                    UmiiEditorSection::Accessories => render_accessories_panel(state, model, &pal, &tab_id).into_any_element(),
                                }),
                        ),
                )
                // 右侧：实时 3D 模型合成画布
                .child(render_umii_viewport(model_doc, &tab_id, model, &pal, state)),
        )
}

/// 渲染分类导航栏胶囊条 (带直观图标与高亮状态)
fn render_section_navigation(
    active: UmiiEditorSection,
    tab_id: &str,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
    state: &AppState,
) -> impl IntoElement {
    let sections = [
        UmiiEditorSection::Basic,
        UmiiEditorSection::Face,
        UmiiEditorSection::Hair,
        UmiiEditorSection::Eyebrows,
        UmiiEditorSection::Eyes,
        UmiiEditorSection::Nose,
        UmiiEditorSection::Mouth,
        UmiiEditorSection::Beard,
        UmiiEditorSection::Accessories,
    ];

    div()
        .flex()
        .items_center()
        .flex_wrap()
        .gap_1()
        .p_2()
        .bg(pal.card_surface)
        .border_b_1()
        .border_color(pal.card_border)
        .children(sections.into_iter().map(|sec| {
            let is_sel = active == sec;
            let m = model.clone();
            let tid = tab_id.to_string();

            div()
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(if is_sel { rgb(0x2563EB) } else { pal.card_hover })
                .border_1()
                .border_color(if is_sel { rgb(0x60A5FA) } else { pal.card_border })
                .cursor_pointer()
                .hover(|s| s.border_color(rgb(0x3B82F6)))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    m.update(cx, |st, cx| {
                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                            if let Some(ref mut u) = t.umii_state {
                                u.active_section = sec;
                            }
                        }
                        cx.notify();
                    });
                })
                .child(div().text_xs().child(sec.icon()))
                .child(
                    div()
                        .text_xs()
                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                        .child(state.t(sec.title_key())),
                )
        }))
}
