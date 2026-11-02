use super::{ActorRef, LinkTargetKind, MapDocumentState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapViewMode {
    Game,
    Context,
    Diagnostic,
}

impl MapViewMode {
    pub const ALL: [Self; 3] = [Self::Game, Self::Context, Self::Diagnostic];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Game => "游戏视图",
            Self::Context => "上下文",
            Self::Diagnostic => "诊断",
        }
    }
}

impl MapDocumentState {
    pub fn set_view_mode(&mut self, mode: MapViewMode) {
        self.view_mode = mode;
        self.show_tags = mode == MapViewMode::Diagnostic;
        self.show_links = mode != MapViewMode::Game;
        self.selected_links_only = mode != MapViewMode::Diagnostic;
        self.show_hidden_perspective = false;
        self.rebuild_instances();
    }

    pub fn actor_kind(&self, reference: ActorRef) -> LinkTargetKind {
        self.actor_kinds
            .get(&reference)
            .copied()
            .unwrap_or(LinkTargetKind::MapObject)
    }

    pub fn category_matches(&self, kind: LinkTargetKind) -> bool {
        self.category_filter.is_none_or(|filter| filter == kind)
    }

    /// A generic LinkTag can reference several systems without becoming one.
    pub fn object_category_matches(&self, reference: ActorRef) -> bool {
        self.category_filter.is_none_or(|filter| {
            self.actor_kind(reference) == filter
                || self
                    .related_kinds
                    .get(&reference)
                    .is_some_and(|mask| mask & (1 << filter.index()) != 0)
        })
    }

    pub fn toggle_tags(&mut self) {
        self.show_tags = !self.show_tags;
        if self.show_tags && self.view_mode == MapViewMode::Game {
            self.view_mode = MapViewMode::Context;
        }
        self.rebuild_instances();
    }

    pub fn toggle_links(&mut self) {
        self.show_links = !self.show_links;
        if self.show_links && self.view_mode == MapViewMode::Game {
            self.view_mode = MapViewMode::Context;
            self.rebuild_instances();
        }
    }

    /// The selected object remains available for editing. Explicit per-object
    /// hiding and disabled map layers still take precedence.
    pub fn object_visible(&self, reference: ActorRef) -> bool {
        if self.hidden_objects.contains(&reference)
            || (matches!(reference, ActorRef::Context(_)) && !self.show_context)
        {
            return false;
        }
        let Some(visual) = self.actor(reference).and_then(|actor| {
            self.assets
                .actors
                .get(uking_library::map::actor_name(actor))
        }) else {
            return false;
        };
        if visual.has_model {
            return self.show_models;
        }
        if self.view_mode == MapViewMode::Game {
            return false;
        }
        if self.selected_ref() == Some(reference) {
            return true;
        }
        self.object_category_matches(reference)
            && (self.show_tags
                || (self.view_mode == MapViewMode::Context
                    && self.context_objects.contains(&reference)))
    }

    /// Report the editor state rather than claiming that an off-screen or
    /// terrain-occluded object has been culled by the renderer.
    pub fn object_display_status(&self, reference: ActorRef) -> &'static str {
        if self.hidden_objects.contains(&reference) {
            return "用户隐藏";
        }
        if matches!(reference, ActorRef::Context(_)) && !self.show_context {
            return "配套图层已关闭";
        }
        if self.loading {
            return "资源加载中";
        }
        let Some(visual) = self.actor(reference).and_then(|actor| {
            self.assets
                .actors
                .get(uking_library::map::actor_name(actor))
        }) else {
            return "资源未加载";
        };
        if visual.has_model && !self.show_models {
            return "模型图层已关闭";
        }
        if !visual.has_model && self.view_mode == MapViewMode::Game {
            return "游戏视图隐藏辅助对象";
        }
        if self.selected_ref() != Some(reference)
            && !visual.has_model
            && !self.object_category_matches(reference)
        {
            return "分类已过滤";
        }
        if !self.object_visible(reference) {
            return "逻辑标记未展开";
        }
        if visual.diagnostic.is_some() {
            return "模型资源缺失 · 占位标记";
        }
        if !visual.has_model {
            return "无模型 · 逻辑标记";
        }
        "可见"
    }
}

#[cfg(test)]
mod tests {
    use super::MapViewMode;
    use crate::model::map_editor::{ActorRef, ActorVisual, LinkTargetKind, MapDocumentState};
    use roead::byml::Byml;
    use std::sync::Arc;
    use uking_library::map::MapFile;

    fn document() -> MapDocumentState {
        let root = Byml::from_text(r#"
Objs:
  - {UnitConfigName: LinkTagAnd, HashId: !u 1, LinksToObj: [{DestUnitHashId: !u 2, DefinitionName: BasicSig}]}
  - {UnitConfigName: CameraTag, HashId: !u 2, LinksToObj: [{DestUnitHashId: !u 3, DefinitionName: BasicSig}]}
  - {UnitConfigName: EventTag, HashId: !u 3}
  - {UnitConfigName: ModelActor, HashId: !u 4}
"#).unwrap();
        let file = MapFile::from_bytes(&root.to_binary(roead::Endian::Little)).unwrap();
        let mut doc = MapDocumentState::empty(file, "display-test", "display-test");
        doc.loading = false;
        for name in ["LinkTagAnd", "CameraTag", "EventTag", "ModelActor"] {
            Arc::make_mut(&mut doc.assets).actors.insert(
                name.into(),
                ActorVisual {
                    meshes: Vec::new(),
                    resource: String::new(),
                    profile: String::new(),
                    has_model: name == "ModelActor",
                    diagnostic: None,
                },
            );
        }
        doc.rebuild_instances();
        doc
    }

    #[test]
    fn context_expands_only_one_hop_and_modes_preserve_document_history() {
        let mut doc = document();
        let original_root = doc.file.root.clone();
        assert_eq!(doc.view_mode, MapViewMode::Context);
        assert!(!doc.object_visible(ActorRef::Current(0)));
        assert!(doc.object_visible(ActorRef::Current(3)));
        doc.select(Some(0));
        assert!(doc.object_visible(ActorRef::Current(0)));
        assert!(doc.object_visible(ActorRef::Current(1)));
        assert!(!doc.object_visible(ActorRef::Current(2)));
        doc.set_view_mode(MapViewMode::Game);
        assert!(!doc.object_visible(ActorRef::Current(0)));
        assert!(!doc.object_visible(ActorRef::Current(1)));
        assert!(doc.object_visible(ActorRef::Current(3)));
        doc.set_view_mode(MapViewMode::Diagnostic);
        assert!(doc.object_visible(ActorRef::Current(2)));
        assert!(!doc.show_hidden_perspective);
        assert!(Arc::ptr_eq(&doc.file.root, &original_root));
        assert!(doc.undo.is_empty());
        assert!(doc.redo.is_empty());
    }

    #[test]
    fn explicit_hidden_and_disabled_layers_take_precedence_over_selection() {
        let mut doc = document();
        doc.select(Some(0));
        doc.toggle_object_visibility(ActorRef::Current(0));
        assert!(!doc.object_visible(ActorRef::Current(0)));
        doc.toggle_object_visibility(ActorRef::Current(0));
        assert!(doc.object_visible(ActorRef::Current(0)));
        doc.select(Some(3));
        doc.show_models = false;
        assert!(!doc.object_visible(ActorRef::Current(3)));
        let paired = doc.file.actors()[1].clone();
        Arc::make_mut(&mut doc.assets).context_actors.push(paired);
        doc.select_ref(Some(ActorRef::Context(0)));
        assert!(doc.object_visible(ActorRef::Context(0)));
        doc.show_context = false;
        assert!(!doc.object_visible(ActorRef::Context(0)));
    }

    #[test]
    fn category_filter_hides_other_helpers_but_keeps_selected_actor_editable() {
        let mut doc = document();
        doc.set_view_mode(MapViewMode::Diagnostic);
        doc.category_filter = Some(LinkTargetKind::Camera);
        doc.rebuild_instances();
        // A connector appears in the relevant filter without being renamed
        // or reclassified as the camera it points to.
        assert!(doc.object_visible(ActorRef::Current(0)));
        assert_eq!(
            doc.actor_kind(ActorRef::Current(0)),
            LinkTargetKind::MapObject
        );
        assert!(doc.object_visible(ActorRef::Current(1)));
        assert!(!doc.object_visible(ActorRef::Current(2)));
        assert!(doc.object_visible(ActorRef::Current(3)));
        doc.select(Some(2));
        assert!(doc.object_visible(ActorRef::Current(2)));
    }

    #[test]
    fn enabling_editor_layers_leaves_game_view_and_expands_context() {
        let mut doc = document();
        doc.select(Some(0));
        doc.set_view_mode(MapViewMode::Game);
        doc.toggle_links();
        assert_eq!(doc.view_mode, MapViewMode::Context);
        assert!(doc.show_links);
        assert!(doc.object_visible(ActorRef::Current(1)));
        assert!(!doc.object_visible(ActorRef::Current(2)));
        doc.set_view_mode(MapViewMode::Game);
        doc.toggle_tags();
        assert_eq!(doc.view_mode, MapViewMode::Context);
        assert!(doc.show_tags);
        assert!(doc.object_visible(ActorRef::Current(2)));
    }

    #[test]
    fn selection_reuses_link_analysis_and_edits_refresh_coordinates_and_profile() {
        let mut doc = document();
        let links = doc.links.clone();
        let kinds = doc.actor_kinds.clone();
        doc.select(Some(0));
        doc.toggle_tags();
        assert!(Arc::ptr_eq(&links, &doc.links));
        assert!(Arc::ptr_eq(&kinds, &doc.actor_kinds));
        let position = uking_library::map::Transform {
            translation: [10., 20., 30.],
            ..Default::default()
        };
        doc.file.set_transform(1, position).unwrap();
        doc.rebuild_instances();
        assert_eq!(doc.links[0].to, Some(position.translation));
        assert!(!Arc::ptr_eq(&links, &doc.links));
        Arc::make_mut(&mut doc.assets)
            .actors
            .get_mut("CameraTag")
            .unwrap()
            .profile = "EventTag".into();
        doc.rebuild_instances();
        assert_eq!(doc.links[0].target_kind, LinkTargetKind::Event);
    }
}
