use super::button;
use crate::{
    model::map_editor::{
        properties::{key, type_name, value_at, value_text},
        ActorRef, LinkKindSource, LinkTargetKind, MapDocumentState, MapInput, PropertyPart,
        PropertyPath, PropertyType,
    },
    ui::{state::AppState, theme::ThemePalette},
};
use gpui::*;
use roead::byml::Byml;
use uking_library::map::{actor_name, hash_id, Transform};

pub fn section(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
    name: &str,
    title: &str,
    body: AnyElement,
) -> AnyElement {
    section_lazy(map, state, model, id, name, title, || body)
}

/// Build an inspector section body only while the section is expanded. This
/// matters for BYML maps with large parameter/link arrays: GPUI still rebuilds
/// the parent view when the inspector scrolls or a field changes.
pub fn section_lazy<F: FnOnce() -> AnyElement>(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
    name: &str,
    title: &str,
    body: F,
) -> AnyElement {
    let pal = state.theme();
    let collapsed = map.collapsed_sections.contains(name);
    let name = name.to_string();
    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .mb_1()
        .child(button(
            format!("{}  {title}", if collapsed { "▸" } else { "▾" }),
            false,
            &pal,
            model,
            id,
            move |m| {
                if !m.collapsed_sections.remove(&name) {
                    m.collapsed_sections.insert(name.clone());
                }
                Ok(())
            },
        ))
        .children((!collapsed).then(|| div().p_2().flex().flex_col().gap_1().child(body())))
        .into_any_element()
}

fn input(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
    field: MapInput,
    text: String,
    path: PropertyPath,
) -> AnyElement {
    let pal = state.theme();
    let editing = map.input == Some(field)
        && (field != MapInput::Property && field != MapInput::PropertyName
            || map.edit_path == path);
    let content = if editing {
        map.input_text.clone()
    } else {
        text.clone()
    };
    let m = model.clone();
    let tid = id.to_string();
    div()
        .min_w_0()
        .flex_1()
        .px_2()
        .py_1()
        .min_h(px(27.))
        .overflow_hidden()
        .rounded_sm()
        .border_1()
        .border_color(if editing {
            pal.primary_accent
        } else {
            pal.card_border
        })
        .bg(pal.card_surface)
        .cursor_text()
        .text_xs()
        .child(crate::ui::components::render_input_text(
            &content, editing, state, model,
        ))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            m.update(cx, |st, cx| {
                if st
                    .active_map()
                    .is_some_and(|m| m.input == Some(field) && m.edit_path == path)
                {
                    return;
                }
                st.clear_active_text_inputs();
                if st
                    .active_map()
                    .is_some_and(|m| m.input.is_some() && m.input_error.is_some())
                {
                    return;
                }
                st.update_map(&tid, |map| {
                    if field == MapInput::Property {
                        map.begin_property(path.clone());
                    } else {
                        map.input = Some(field);
                        map.input_text = text.clone();
                        map.edit_path = path.clone();
                        map.input_error = None;
                    }
                    Ok(())
                });
                st.text_input_cursor = text.chars().count();
                st.text_input_select_anchor = Some(0);
                cx.notify();
            });
        })
        .into_any_element()
}
fn muted(text: impl Into<SharedString>, pal: &ThemePalette) -> AnyElement {
    div()
        .text_xs()
        .text_color(pal.text_secondary)
        .child(text.into())
        .into_any_element()
}

fn add_controls(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
    parent: PropertyPath,
) -> AnyElement {
    let pal = state.theme();
    let parent_button = parent.clone();
    let opened =
        map.add_property_parent == parent && map.collapsed_sections.contains("add-property");
    let mut container = div().flex().flex_col().gap_1().mt_1().child(button(
        "+ 添加属性",
        false,
        &pal,
        model,
        id,
        move |m| {
            let was_open = m.add_property_parent == parent_button
                && m.collapsed_sections.contains("add-property");
            m.add_property_parent = parent_button.clone();
            if was_open {
                m.collapsed_sections.remove("add-property");
            } else {
                m.collapsed_sections.insert("add-property".into());
            }
            Ok(())
        },
    ));
    if opened {
        let is_array = map
            .selected_actor()
            .and_then(|a| value_at(a, &parent))
            .is_some_and(|v| matches!(v, Byml::Array(_)));
        if !is_array {
            container = container.child(input(
                map,
                state,
                model,
                id,
                MapInput::NewPropertyName,
                map.new_property_name.clone(),
                vec![],
            ));
        }
        container = container.child(div().flex().flex_wrap().gap_1().children(
            PropertyType::ALL.into_iter().map(|kind| {
                button(
                    kind.name(),
                    map.new_property_type == kind,
                    &pal,
                    model,
                    id,
                    move |m| {
                        m.new_property_type = kind;
                        Ok(())
                    },
                )
            }),
        ));
        container = container.child(button("添加", false, &pal, model, id, move |m| {
            let name = m.new_property_name.clone();
            m.add_property(&parent, &name, m.new_property_type)?;
            m.collapsed_sections.remove("add-property");
            Ok(())
        }));
    }
    container.into_any_element()
}

fn properties(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
    path: PropertyPath,
    value: &Byml,
    readonly: bool,
    depth: usize,
) -> AnyElement {
    let pal = state.theme();
    let mut result = div().flex().flex_col().gap_1().min_w_0();
    if depth > 32 {
        return muted("嵌套超过 32 层，请使用 BYML 代码编辑器", &pal);
    }
    let children: Vec<(PropertyPart, &Byml)> = match value {
        Byml::Map(m) => {
            let mut entries: Vec<_> = m.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            entries
                .into_iter()
                .filter(|(k, _)| {
                    !path.is_empty()
                        || !["Translate", "Rotate", "Scale", "!Parameters", "LinksToObj"]
                            .contains(&k.as_str())
                })
                .map(|(k, v)| (key(k), v))
                .collect()
        }
        Byml::Array(a) => a
            .iter()
            .enumerate()
            .map(|(i, v)| (PropertyPart::Index(i), v))
            .collect(),
        _ => return muted("此属性不是 Map 或 Array", &pal),
    };
    for (part, v) in children {
        let mut child_path = path.clone();
        child_path.push(part.clone());
        let label = match &part {
            PropertyPart::Key(k) => k.clone(),
            PropertyPart::Index(i) => format!("[{i}]"),
        };
        let protected = (path.is_empty()
            && ["HashId", "UnitConfigName", "SRTHash"].contains(&label.as_str()))
            || (matches!(path.as_slice(),[PropertyPart::Key(k),PropertyPart::Index(_)] if k=="LinksToObj")
                && ["DefinitionName", "DestUnitHashId"].contains(&label.as_str()));
        let nested = matches!(v, Byml::Map(_) | Byml::Array(_));
        let mut row = div().flex().items_center().gap_1().min_w_0();
        let label_el = div()
            .w(px(if protected { 145. } else { 115. }))
            .flex_shrink_0()
            .overflow_hidden()
            .text_xs()
            .child(div().child(label.clone()))
            .child(
                div()
                    .text_size(px(9.))
                    .text_color(pal.text_secondary)
                    .child(type_name(v)),
            );
        if nested {
            let section_id = format!("property:{child_path:?}");
            let collapsed = !map.collapsed_sections.contains(&section_id);
            let toggle = section_id.clone();
            row = row.child(button(
                format!(
                    "{} {label} · {}",
                    if collapsed { "▸" } else { "▾" },
                    match v {
                        Byml::Map(m) => m.len(),
                        Byml::Array(a) => a.len(),
                        _ => 0,
                    }
                ),
                false,
                &pal,
                model,
                id,
                move |m| {
                    if !m.collapsed_sections.remove(&toggle) {
                        m.collapsed_sections.insert(toggle.clone());
                    }
                    Ok(())
                },
            ));
            if !readonly {
                if matches!(part, PropertyPart::Key(_)) {
                    let p = child_path.clone();
                    let name = label.clone();
                    row = row.child(button("改名", false, &pal, model, id, move |m| {
                        m.edit_path = p.clone();
                        m.input = Some(MapInput::PropertyName);
                        m.input_text = name.clone();
                        Ok(())
                    }));
                }
                let p = child_path.clone();
                row = row.child(button("×", false, &pal, model, id, move |m| {
                    m.remove_property(&p)
                }));
            }
            result = result.child(row);
            if map.input == Some(MapInput::PropertyName) && map.edit_path == child_path {
                result = result.child(input(
                    map,
                    state,
                    model,
                    id,
                    MapInput::PropertyName,
                    label,
                    child_path.clone(),
                ));
            }
            if !collapsed {
                result = result.child(
                    div()
                        .pl_2()
                        .border_l_1()
                        .border_color(pal.card_border)
                        .child(properties(
                            map,
                            state,
                            model,
                            id,
                            child_path,
                            v,
                            readonly,
                            depth + 1,
                        )),
                );
            }
            continue;
        }
        row = row.child(label_el);
        if readonly {
            row = row.child(muted(value_text(v), &pal));
        } else if let Byml::Bool(enabled) = v {
            let p = child_path.clone();
            let next = !*enabled;
            row = row.child(button(
                if *enabled { "☑" } else { "☐" },
                false,
                &pal,
                model,
                id,
                move |m| m.set_property(&p, Byml::Bool(next)),
            ));
        } else {
            row = row.child(input(
                map,
                state,
                model,
                id,
                MapInput::Property,
                value_text(v),
                child_path.clone(),
            ));
        }
        if !readonly && !protected {
            if matches!(part, PropertyPart::Key(_)) {
                let p = child_path.clone();
                let name = label.clone();
                row = row.child(button("改名", false, &pal, model, id, move |m| {
                    m.edit_path = p.clone();
                    m.input = Some(MapInput::PropertyName);
                    m.input_text = name.clone();
                    Ok(())
                }));
            }
            let p = child_path.clone();
            row = row.child(button("×", false, &pal, model, id, move |m| {
                m.remove_property(&p)
            }));
        }
        result = result.child(row);
        if map.input == Some(MapInput::PropertyName) && map.edit_path == child_path {
            result = result.child(input(
                map,
                state,
                model,
                id,
                MapInput::PropertyName,
                label,
                child_path,
            ));
        }
    }
    if !readonly {
        result = result.child(add_controls(map, state, model, id, path));
    }
    result.into_any_element()
}

fn transforms(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
    actor: &Byml,
    readonly: bool,
) -> AnyElement {
    let pal = state.theme();
    let t = Transform::from_actor(actor);
    let mut body = div().flex().flex_col().gap_1();
    for (kind, label, values) in [
        (0, "平移", t.translation),
        (1, "旋转 °", t.rotation.map(f32::to_degrees)),
        (2, "缩放", t.scale),
    ] {
        let mut row = div()
            .flex()
            .items_center()
            .gap_1()
            .child(div().w(px(42.)).flex_shrink_0().text_xs().child(label));
        for axis in 0..3 {
            row = row.child(
                div()
                    .text_size(px(10.))
                    .text_color([rgb(0xED7C73), rgb(0x8DC87A), rgb(0x76A9EA)][axis])
                    .child(["X", "Y", "Z"][axis]),
            );
            row = row.child(if readonly {
                muted(format!("{:.3}", values[axis]), &pal)
            } else {
                input(
                    map,
                    state,
                    model,
                    id,
                    MapInput::Transform(kind, axis),
                    values[axis].to_string(),
                    vec![],
                )
            });
        }
        body = body.child(row);
    }
    body = body.child(
        div()
            .flex()
            .flex_wrap()
            .gap_1()
            .child(button("聚焦 F", false, &pal, model, id, |m| {
                m.focus_selected();
                Ok(())
            }))
            .children((!readonly).then(|| {
                button("吸附到地形", false, &pal, model, id, |m| {
                    m.snap_to_terrain()
                })
            }))
            .children(
                (!readonly).then(|| button("复制", false, &pal, model, id, |m| m.duplicate())),
            )
            .children((!readonly).then(|| button("删除", false, &pal, model, id, |m| m.delete()))),
    );
    body.into_any_element()
}

fn jump_button(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
    target: Option<ActorRef>,
    hash: u32,
) -> AnyElement {
    let pal = state.theme();
    let label = target
        .and_then(|r| map.actor(r))
        .map(|a| {
            format!(
                "{} · {hash:08X}{}",
                actor_name(a),
                if matches!(target, Some(ActorRef::Context(_))) {
                    " · 配套"
                } else {
                    ""
                }
            )
        })
        .unwrap_or_else(|| format!("⚠ 未解析 · {hash:08X}"));
    if target.is_none() {
        return muted(label, &pal);
    }
    button(label, false, &pal, model, id, move |m| {
        m.select_ref(target);
        m.focus_selected();
        Ok(())
    })
}
fn link_kind_badge(kind: LinkTargetKind, pal: &ThemePalette) -> AnyElement {
    let (background, foreground) = match kind {
        LinkTargetKind::MapObject => (rgba(0x3A5366AA), rgb(0xB8D8EA)),
        LinkTargetKind::Camera => (rgba(0x7354AFAA), rgb(0xE4D2FF)),
        LinkTargetKind::Event => (rgba(0xA4603FAA), rgb(0xFFD3B8)),
        LinkTargetKind::Weather => (rgba(0x287C82AA), rgb(0xB8F2F2)),
        LinkTargetKind::Audio => (rgba(0x6D6A34AA), rgb(0xF5E7A4)),
        LinkTargetKind::Physics => (rgba(0x3F774FAA), rgb(0xC7F0C6)),
        LinkTargetKind::External => (rgba(0x5D6470AA), rgb(0xD3D8E0)),
        LinkTargetKind::Unknown => (rgba(0x4A4D52AA), pal.text_secondary),
    };
    div()
        .px_1()
        .py(px(1.))
        .rounded_sm()
        .bg(background)
        .text_color(foreground)
        .text_size(px(10.))
        .child(format!("{} {}", kind.symbol(), kind.label()))
        .into_any_element()
}

fn links(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
    incoming: bool,
    readonly: bool,
) -> AnyElement {
    let pal = state.theme();
    let Some(reference) = map.selected_ref() else {
        return div().into_any_element();
    };
    let mut body = div().flex().flex_col().gap_2();
    let mut visible_links: Vec<_> = map.object_links(reference, incoming).collect();
    let count = visible_links.len();
    visible_links.sort_by_key(|link| (link.target_kind.index(), link.index));
    let mut kind_counts = [0usize; 8];
    for link in &visible_links {
        kind_counts[link.target_kind.index()] += 1;
    }
    if count > 0 {
        body = body.child(div().flex().flex_wrap().gap_1().children(
            LinkTargetKind::ALL.into_iter().filter_map(|kind| {
                let count = kind_counts[kind.index()];
                if count == 0 {
                    return None;
                }
                Some(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(link_kind_badge(kind, &pal))
                        .child(
                            div()
                                .text_size(px(10.))
                                .text_color(pal.text_secondary)
                                .child(count.to_string()),
                        )
                        .into_any_element(),
                )
            }),
        ));
    }
    for link in visible_links {
        let target = if incoming {
            Some(link.source)
        } else {
            link.target
        };
        let hash = if incoming {
            map.actor(link.source).and_then(hash_id).unwrap_or(0)
        } else {
            link.destination
        };
        let mut item = div()
            .flex()
            .flex_col()
            .gap_1()
            .p_1()
            .border_b_1()
            .border_color(pal.card_border)
            .child(link_kind_badge(link.target_kind, &pal))
            .child(
                div()
                    .text_xs()
                    .text_color(pal.primary_accent)
                    .child(format!(
                        "{} {}",
                        if incoming { "←" } else { "→" },
                        link.definition
                    )),
            )
            .child(jump_button(map, state, model, id, target, hash))
            .child(muted(
                format!(
                    "{} · 分类依据：{}",
                    if link.target.is_some() {
                        "地图目标已解析"
                    } else {
                        "目标未解析（可能位于其他地图）"
                    },
                    link.kind_source.label()
                ),
                &pal,
            ));
        if link.target.is_none() && link.kind_source == LinkKindSource::Definition {
            item = item.child(muted("仅推断用途，尚未确认系统绑定。", &pal));
        }
        if !incoming && !readonly {
            let path = vec![key("LinksToObj"), PropertyPart::Index(link.index)];
            let section_key = format!("link-edit-{:?}-{}", link.source, link.index);
            let expanded = map.collapsed_sections.contains(&section_key);
            item = item.child(button(
                if expanded {
                    "▾ 收起链接参数"
                } else {
                    "▸ 编辑链接参数"
                },
                false,
                &pal,
                model,
                id,
                move |m| {
                    if !m.collapsed_sections.remove(&section_key) {
                        m.collapsed_sections.insert(section_key.clone());
                    }
                    Ok(())
                },
            ));
            if let Some(value) = map
                .selected_actor()
                .and_then(|a| value_at(a, &path))
                .filter(|_| expanded)
            {
                item = item.child(properties(
                    map,
                    state,
                    model,
                    id,
                    path.clone(),
                    value,
                    false,
                    0,
                ));
            }
            item = item.child(button(
                "移除此链接",
                false,
                &pal,
                model,
                id,
                move |m| m.remove_property(&path),
            ));
        }
        body = body.child(item);
    }
    if count == 0 {
        body = body.child(muted(
            if incoming {
                "无传入链接"
            } else {
                "无传出链接"
            },
            &pal,
        ));
    }
    if !incoming && !readonly {
        body = body
            .child(div().mt_1().text_xs().child("添加链接"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(muted("类型", &pal))
                    .child(input(
                        map,
                        state,
                        model,
                        id,
                        MapInput::LinkDefinition,
                        map.new_link_definition.clone(),
                        vec![],
                    )),
            )
            .child(button(
                "选择链接类型…",
                false,
                &pal,
                model,
                id,
                |m| {
                    if !m.collapsed_sections.remove("link-types") {
                        m.collapsed_sections.insert("link-types".into());
                    }
                    Ok(())
                },
            ));
        if map.collapsed_sections.contains("link-types") {
            const TYPES: &[&str] = &[
                "-AxisX",
                "-AxisY",
                "-AxisZ",
                "AreaCol",
                "AxisX",
                "AxisY",
                "AxisZ",
                "BAndSCs",
                "BAndSLimitAngYCs",
                "BasicSig",
                "BasicSigOnOnly",
                "ChangeAtnSig",
                "CogWheelCs",
                "CopyWaitRevival",
                "Create",
                "DeadUp",
                "Delete",
                "DemoMember",
                "FixedCs",
                "ForSale",
                "ForbidAttention",
                "Freeze",
                "GimmickSuccess",
                "HingeCs",
                "LifeZero",
                "LimitHingeCs",
                "ModelBind",
                "MtxCopyCreate",
                "OffWaitRevival",
                "PhysSystemGroup",
                "PlacementLOD",
                "PulleyCs",
                "RackAndPinionCs",
                "Recreate",
                "Reference",
                "Remains",
                "SensorBlind",
                "SliderCs",
                "Stable",
                "StackLink",
                "SyncLink",
                "VelocityControl",
            ];
            body = body.child(div().flex().flex_wrap().gap_1().children(TYPES.iter().map(
                |&kind| {
                    button(
                        kind,
                        map.new_link_definition == kind,
                        &pal,
                        model,
                        id,
                        move |m| {
                            m.new_link_definition = kind.into();
                            m.collapsed_sections.remove("link-types");
                            Ok(())
                        },
                    )
                },
            )));
        }
        body = body
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(muted("目标", &pal))
                    .child(input(
                        map,
                        state,
                        model,
                        id,
                        MapInput::LinkTarget,
                        map.new_link_target.clone(),
                        vec![],
                    )),
            )
            .child(muted(
                "HashId：十进制或 0x 十六进制；目标可在配套地图",
                &pal,
            ))
            .child(button("+ 添加链接", false, &pal, model, id, |m| {
                m.add_link()
            }));
    }
    body.into_any_element()
}

fn systems(map: &MapDocumentState, state: &AppState) -> AnyElement {
    let pal = state.theme();
    let mut body = div().flex().flex_col().gap_2();
    for reference in map
        .selected_ref()
        .into_iter()
        .flat_map(|source| map.object_system_references(source))
    {
        body = body.child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(link_kind_badge(reference.kind, &pal))
                .child(muted(reference.parameter.clone(), &pal))
                .child(div().text_xs().child(reference.value.clone()))
                .children(
                    reference
                        .entry_point
                        .as_ref()
                        .map(|entry| muted(format!("入口：{entry}"), &pal)),
                ),
        );
    }
    body.child(muted(
        "来自对象参数；运行时系统绑定尚未验证。地图对象链接单独列于下方。",
        &pal,
    ))
    .into_any_element()
}

pub fn render(
    map: &MapDocumentState,
    state: &AppState,
    model: &Entity<AppState>,
    id: &str,
) -> AnyElement {
    let pal = state.theme();
    let readonly = map.selected_context.is_some();
    let mut panel = div()
        .id("map-inspector")
        .w(px(map.inspector_width))
        .flex_shrink_0()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(pal.panel_surface)
        .child(
            div()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(pal.card_border)
                .font_weight(FontWeight::BOLD)
                .text_xs()
                .child("属性"),
        );
    let mut content = div()
        .id("map-inspector-scroll")
        .flex_1()
        .min_h_0()
        .p_2()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap_1();
    if let Some(error) = map.load_error.as_ref().or(map.input_error.as_ref()) {
        content = content.child(
            div()
                .p_2()
                .text_xs()
                .text_color(rgb(0xED8B71))
                .child(error.clone()),
        );
    }
    let Some(actor) = map.selected_actor() else {
        return panel
            .child(content.child(muted(
                "选择对象以查看属性、参数与链接。\n逻辑对象可从大纲选择。",
                &pal,
            )))
            .into_any_element();
    };
    content = content.child(
        div()
            .px_1()
            .py_1()
            .text_sm()
            .font_weight(FontWeight::BOLD)
            .child(actor_name(actor).to_string()),
    );
    let source = if readonly {
        map.assets.context_path.as_deref().unwrap_or("配套地图")
    } else {
        &map.scene.file_path
    };
    let file = source.rsplit(['\\', '/']).next().unwrap_or(source);
    content = content.child(muted(
        format!("{file} · HashId {:08X}", hash_id(actor).unwrap_or(0)),
        &pal,
    ));
    if let Some(reference) = map.selected_ref() {
        content = content.child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(link_kind_badge(map.actor_kind(reference), &pal))
                .child(muted(map.object_display_status(reference), &pal)),
        );
    }
    if readonly {
        let path = map.assets.context_path.clone();
        let hash = hash_id(actor);
        let m = model.clone();
        content = content.child(
            div()
                .p_2()
                .text_xs()
                .text_color(pal.primary_accent)
                .cursor_pointer()
                .child("配套地图 · 只读，点击打开所属地图编辑 ↗")
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    if let Some(path) = &path {
                        m.update(cx, |st, cx| {
                            st.clear_active_text_inputs();
                            st.load_file_by_path(path);
                            if let Some(map) = st.active_map_mut() {
                                let r = hash.and_then(|h| map.resolve_hash(h));
                                map.select_ref(r);
                                if !map.loading {
                                    map.focus_selected();
                                }
                            }
                            cx.notify();
                        });
                    }
                }),
        );
    }
    content = content
        .children(
            map.selected_ref()
                .is_some_and(|source| map.object_system_references(source).len() > 0)
                .then(|| {
                    section_lazy(
                        map,
                        state,
                        model,
                        id,
                        "system-references",
                        "系统引用 / 参数绑定",
                        || systems(map, state),
                    )
                }),
        )
        .child(section_lazy(
            map,
            state,
            model,
            id,
            "transform",
            "变换",
            || transforms(map, state, model, id, actor, readonly),
        ))
        .child(section_lazy(
            map,
            state,
            model,
            id,
            "object",
            "Obj / 对象",
            || properties(map, state, model, id, vec![], actor, readonly, 0),
        ));
    let parameters = actor.as_map().ok().and_then(|m| m.get("!Parameters"));
    let mut param_body = div().flex().flex_col().gap_1();
    if !readonly {
        let copy_model = model.clone();
        let paste_model = model.clone();
        let tid = id.to_string();
        param_body = param_body.child(
            div()
                .flex()
                .gap_1()
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .bg(pal.card_surface)
                        .cursor_pointer()
                        .child("复制参数")
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let copy = copy_model.update(cx, |st, _| {
                                st.clear_active_text_inputs();
                                if st
                                    .active_map()
                                    .is_some_and(|m| m.input.is_some() && m.input_error.is_some())
                                {
                                    return None;
                                }
                                Some(
                                    st.active_map()
                                        .and_then(|m| m.selected_actor())
                                        .and_then(|a| value_at(a, &[key("!Parameters")]))
                                        .cloned()
                                        .unwrap_or_else(|| Byml::Map(Default::default())),
                                )
                            });
                            if let Some(copy) = copy {
                                cx.write_to_clipboard(ClipboardItem::new_string(copy.to_text()));
                            }
                        }),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .bg(pal.card_surface)
                        .cursor_pointer()
                        .child("粘贴参数")
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let text = cx.read_from_clipboard().and_then(|i| i.text());
                            paste_model.update(cx, |st, cx| {
                                st.clear_active_text_inputs();
                                if st
                                    .active_map()
                                    .is_some_and(|m| m.input.is_some() && m.input_error.is_some())
                                {
                                    return;
                                }
                                st.update_map(&tid, |map| {
                                    let result = text
                                        .as_ref()
                                        .ok_or_else(|| "剪贴板没有文本".to_string())
                                        .and_then(|s| Byml::from_text(s).map_err(|e| e.to_string()))
                                        .and_then(|v| map.replace_parameters(v));
                                    map.input_error = result.as_ref().err().cloned();
                                    result
                                });
                                cx.notify();
                            });
                        }),
                ),
        );
    }
    if let Some(parameters) = parameters {
        if !map.collapsed_sections.contains("parameters") {
            param_body = param_body.child(properties(
                map,
                state,
                model,
                id,
                vec![key("!Parameters")],
                parameters,
                readonly,
                0,
            ));
        }
    } else if readonly {
        param_body = param_body.child(muted("无参数", &pal));
    } else {
        param_body = param_body.child(button("+ 创建参数表", false, &pal, model, id, |m| {
            m.replace_parameters(Byml::Map(Default::default()))
        }));
    }
    content = content.child(section_lazy(
        map,
        state,
        model,
        id,
        "parameters",
        "参数",
        || param_body.into_any_element(),
    ));
    let incoming = map
        .selected_ref()
        .map_or(0, |source| map.object_links(source, true).len());
    let outgoing = map
        .selected_ref()
        .map_or(0, |source| map.object_links(source, false).len());
    content = content
        .child(section_lazy(
            map,
            state,
            model,
            id,
            "incoming",
            &format!("传入链接 ({incoming})"),
            || links(map, state, model, id, true, readonly),
        ))
        .child(section_lazy(
            map,
            state,
            model,
            id,
            "outgoing",
            &format!("传出链接 ({outgoing})"),
            || links(map, state, model, id, false, readonly),
        ));
    content = content
        .child(section_lazy(
            map,
            state,
            model,
            id,
            "source",
            "模型来源 / 资源诊断",
            || {
                let mut diagnostics = div().flex().flex_col().gap_1();
                if let Some(visual) = map.assets.actors.get(actor_name(actor)) {
                    diagnostics = diagnostics.child(muted(visual.resource.clone(), &pal));
                    if let Some(error) = &visual.diagnostic {
                        diagnostics = diagnostics.child(muted(error.clone(), &pal));
                    }
                }
                for warning in &map.assets.warnings {
                    diagnostics = diagnostics.child(muted(warning.clone(), &pal));
                }
                diagnostics.into_any_element()
            },
        ))
        .child(muted(
            "Enter 应用 · Esc 取消 · Ctrl+Z 撤销\n布尔值点击切换；整数保留原始类型。",
            &pal,
        ));
    panel = panel.child(content);
    panel.into_any_element()
}
