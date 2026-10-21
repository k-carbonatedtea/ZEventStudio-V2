use crate::model::plugin::{PluginInstance, PluginKind};
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use gpui::*;

/// 渲染单个插件的详情卡片组件
pub fn render_plugin_card(
    plugin: &PluginInstance,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let is_enabled = plugin.is_enabled();
    let plugin_id = plugin.id().to_string();
    let is_mcp = plugin.manifest.kind == PluginKind::Mcp || plugin.id() == "zevent-mcp-bridge";

    let model_toggle = model.clone();
    let id_for_toggle = plugin_id.clone();

    div()
        .w_full()
        .flex()
        .flex_col()
        .p_3()
        .rounded_lg()
        .bg(pal.card_surface)
        .border_1()
        .border_color(if is_enabled {
            pal.panel_border
        } else {
            pal.panel_border.opacity(0.4)
        })
        .gap_2()

        // 1. 卡片头部：图标 + 标题/版本 + 启停开关
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                // 左侧 图标与信息
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(20.0))
                                .child(plugin.manifest.icon.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .child(
                                            div()
                                                .text_size(px(13.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_enabled {
                                                    pal.text_primary
                                                } else {
                                                    pal.text_secondary
                                                })
                                                .child(plugin.manifest.name.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.0))
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(pal.app_bg)
                                                .text_color(pal.primary_accent)
                                                .child(format!("v{}", plugin.manifest.version)),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(pal.text_secondary)
                                        .child(format!("by {}", plugin.manifest.author)),
                                ),
                        ),
                )
                // 右侧 启停状态胶囊按钮
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .bg(if is_enabled {
                            pal.primary_accent.opacity(0.15)
                        } else {
                            pal.app_bg
                        })
                        .text_color(if is_enabled {
                            pal.primary_accent
                        } else {
                            pal.text_secondary
                        })
                        .border_1()
                        .border_color(if is_enabled {
                            pal.primary_accent.opacity(0.4)
                        } else {
                            pal.panel_border
                        })
                        .hover(|s| s.opacity(0.8))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let pid = id_for_toggle.clone();
                            model_toggle.update(cx, |st, cx| {
                                st.toggle_plugin(&pid);
                                cx.notify();
                            });
                        })
                        .child(if is_enabled {
                            state.t("plugins.status_enabled")
                        } else {
                            state.t("plugins.status_disabled")
                        }),
                ),
        )

        // 2. 插件描述文本
        .child(
            div()
                .text_size(px(11.5))
                .text_color(pal.text_secondary)
                .line_height(relative(1.3))
                .child(plugin.manifest.description.clone()),
        )

        // 3. MCP 专属快速操作区 (如果为 MCP 插件)
        .children(is_mcp.then(|| {
            render_mcp_special_box(state, model)
        }))

        // 4. 插件声明的能力标签 (Commands / Tools)
        .child(render_contributes_summary(plugin, state))
}

/// 渲染 MCP 协议插件专属操作控制台
fn render_mcp_special_box(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let server_running = state.api_server_running;
    let model_claude = model.clone();
    let model_cursor = model.clone();
    let model_server = model.clone();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_2p5()
        .rounded_md()
        .bg(pal.app_bg)
        .border_1()
        .border_color(pal.panel_border)
        // 运行状态指示器
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .size(px(7.0))
                                .rounded_full()
                                .bg(if server_running {
                                    rgb(0x10B981) // 绿色正常运行
                                } else {
                                    rgb(0x9CA3AF) // 灰色未启动
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_primary)
                                .child(if server_running {
                                    format!("{}: 127.0.0.1:{}", state.t("plugins.api_server"), state.api_server_port)
                                } else {
                                    format!("{}: {}", state.t("plugins.api_server"), state.t("plugins.api_server_stopped"))
                                }),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .cursor_pointer()
                        .text_size(px(10.5))
                        .bg(if server_running {
                            rgb(0xEF4444).opacity(0.15)
                        } else {
                            pal.primary_accent.opacity(0.15)
                        })
                        .text_color(if server_running {
                            rgb(0xEF4444)
                        } else {
                            pal.primary_accent
                        })
                        .hover(|s| s.opacity(0.8))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let is_running = model_server.read(cx).api_server_running;
                            if is_running {
                                crate::ui::state_plugins::stop_plugin_bridge_server(&model_server, cx);
                            } else {
                                crate::ui::state_plugins::start_plugin_bridge_server(&model_server, cx);
                            }
                        })
                        .child(if server_running {
                            state.t("plugins.mcp_stop")
                        } else {
                            state.t("plugins.mcp_start")
                        }),
                ),
        )
        // 复制配置按钮组
        .child(
            div()
                .flex()
                .gap_1p5()
                // 复制 Claude Desktop 配置
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py_1()
                        .rounded_md()
                        .cursor_pointer()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_claude.update(cx, |st, cx| {
                                let config = st.generate_claude_config();
                                cx.write_to_clipboard(ClipboardItem::new_string(config));
                                st.toast = Some(Toast::success(st.t("plugins.copied_toast")));
                                cx.notify();
                            });
                        })
                        .child(state.t("plugins.copy_claude")),
                )
                // 复制 Cursor 配置
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py_1()
                        .rounded_md()
                        .cursor_pointer()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_cursor.update(cx, |st, cx| {
                                let config = st.generate_cursor_config();
                                cx.write_to_clipboard(ClipboardItem::new_string(config));
                                st.toast = Some(Toast::success(st.t("plugins.copied_toast")));
                                cx.notify();
                            });
                        })
                        .child(state.t("plugins.copy_cursor")),
                ),
        )
}

/// 渲染贡献能力徽标摘要
fn render_contributes_summary(plugin: &PluginInstance, state: &AppState) -> impl IntoElement {
    let pal = state.theme();
    let commands_count = plugin.manifest.contributes.commands.len();
    let tools_count = plugin.manifest.contributes.tools.len();

    div()
        .flex()
        .items_center()
        .gap_2()
        .pt_1()
        .text_size(px(10.5))
        .text_color(pal.text_secondary)
        .children((commands_count > 0).then(|| {
            div()
                .px_1p5()
                .py_0p5()
                .rounded(px(2.0))
                .bg(pal.app_bg)
                .child(format!("{} {} {}", commands_count, state.t("plugins.commands"), ""))
        }))
        .children((tools_count > 0).then(|| {
            div()
                .px_1p5()
                .py_0p5()
                .rounded(px(2.0))
                .bg(pal.app_bg)
                .child(format!("{} {} {}", tools_count, state.t("plugins.tools"), ""))
        }))
}
