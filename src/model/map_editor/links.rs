use super::MapDocumentState;
use roead::byml::Byml;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use uking_library::map::{hash_id, Transform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorRef {
    Current(usize),
    Context(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkTargetKind {
    MapObject,
    Camera,
    Event,
    Weather,
    Audio,
    Physics,
    External,
    Unknown,
}

impl LinkTargetKind {
    pub const ALL: [Self; 8] = [
        Self::MapObject,
        Self::Camera,
        Self::Event,
        Self::Weather,
        Self::Audio,
        Self::Physics,
        Self::External,
        Self::Unknown,
    ];
    pub const fn index(self) -> usize {
        match self {
            Self::MapObject => 0,
            Self::Camera => 1,
            Self::Event => 2,
            Self::Weather => 3,
            Self::Audio => 4,
            Self::Physics => 5,
            Self::External => 6,
            Self::Unknown => 7,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::MapObject => "地图对象",
            Self::Camera => "镜头",
            Self::Event => "事件",
            Self::Weather => "天气",
            Self::Audio => "音频",
            Self::Physics => "物理",
            Self::External => "外部系统",
            Self::Unknown => "未知类型",
        }
    }

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::MapObject => "□",
            Self::Camera => "△",
            Self::Event => "◇",
            Self::Weather => "○",
            Self::Audio => "♫",
            Self::Physics => "⬡",
            Self::External => "◎",
            Self::Unknown => "?",
        }
    }

    pub const fn color(self) -> u32 {
        match self {
            Self::MapObject => 0x58A6FF,
            Self::Camera => 0xB98CFF,
            Self::Event => 0xFFB07C,
            Self::Weather => 0x66E0E0,
            Self::Audio => 0xF2D66D,
            Self::Physics => 0x83D483,
            Self::External => 0xD3D8E0,
            Self::Unknown => 0xB8BEC8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKindSource {
    Profile,
    ActorName,
    Parameters,
    Definition,
    MapObject,
    Unknown,
}

impl LinkKindSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Profile => "Actor Profile",
            Self::ActorName => "Actor 名称推断",
            Self::Parameters => "对象参数",
            Self::Definition => "链接类型推断",
            Self::MapObject => "已解析地图对象",
            Self::Unknown => "无分类依据",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemReference {
    pub source: ActorRef,
    pub kind: LinkTargetKind,
    pub parameter: String,
    pub value: String,
    pub entry_point: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectLink {
    pub source: ActorRef,
    pub target: Option<ActorRef>,
    pub target_kind: LinkTargetKind,
    pub kind_source: LinkKindSource,
    pub index: usize,
    pub destination: u32,
    pub definition: String,
    pub from: [f32; 3],
    pub to: Option<[f32; 3]>,
}

pub fn link_destination(link: &Byml) -> Option<u32> {
    match link.as_map().ok()?.get("DestUnitHashId")? {
        Byml::U32(v) => Some(*v),
        Byml::I32(v) => Some(*v as u32),
        _ => None,
    }
}

fn semantic_kind(value: &str) -> Option<LinkTargetKind> {
    // Match words, not arbitrary substrings: Terrain is not Rain, Window is
    // not Wind, and an enemy holding a thunder weapon is not a weather system.
    let bytes = value.as_bytes();
    let mut separated = String::with_capacity(value.len() + 8);
    for (i, &byte) in bytes.iter().enumerate() {
        if byte.is_ascii_uppercase()
            && i > 0
            && (bytes[i - 1].is_ascii_lowercase()
                || (bytes[i - 1].is_ascii_uppercase()
                    && bytes.get(i + 1).is_some_and(u8::is_ascii_lowercase)))
        {
            separated.push(' ');
        }
        separated.push(if byte.is_ascii_alphanumeric() {
            byte.to_ascii_lowercase() as char
        } else {
            ' '
        });
    }
    let has = |terms: &[&str]| {
        separated
            .split_whitespace()
            .any(|word| terms.contains(&word))
    };
    if has(&["camera", "viewpoint", "viewtarget"]) {
        Some(LinkTargetKind::Camera)
    } else if has(&["weather", "climate"]) {
        Some(LinkTargetKind::Weather)
    } else if has(&["event", "eventflow", "demo", "cutscene", "timeline"]) {
        Some(LinkTargetKind::Event)
    } else if has(&["sound", "audio", "bgm", "voice", "seemit", "envse", "music"]) {
        Some(LinkTargetKind::Audio)
    } else if has(&["physics", "phys", "collision", "navmesh", "airwall"])
        || matches!(
            value,
            "BoxWater"
                | "AscendingCurrent"
                | "AirWall"
                | "NavMesh"
                | "FixedCs"
                | "HingeCs"
                | "LimitHingeCs"
                | "SliderCs"
                | "PulleyCs"
                | "CogWheelCs"
                | "RackAndPinionCs"
                | "BAndSCs"
                | "BAndSLimitAngYCs"
                | "PhysSystemGroup"
                | "VelocityControl"
        )
    {
        Some(LinkTargetKind::Physics)
    } else if has(&["global", "system", "manager", "service", "external"]) {
        Some(LinkTargetKind::External)
    } else {
        None
    }
}

fn parameter_kind(key: &str) -> Option<LinkTargetKind> {
    match key {
        "EventFlowName" | "EventFlowFileName" | "DemoName" => Some(LinkTargetKind::Event),
        "CameraName" | "CameraPreset" | "CameraPresetName" => Some(LinkTargetKind::Camera),
        "WeatherType" | "WeatherName" | "WeatherPreset" | "ClimateName" => {
            Some(LinkTargetKind::Weather)
        }
        "BgmName" | "SoundName" | "AudioEventName" => Some(LinkTargetKind::Audio),
        _ => None,
    }
}

fn parameter_references(reference: ActorRef, actor: &Byml) -> Vec<SystemReference> {
    let Some(parameters) = actor
        .as_map()
        .ok()
        .and_then(|m| m.get("!Parameters"))
        .and_then(|p| p.as_map().ok())
    else {
        return Vec::new();
    };
    let entry_point = [
        "EventFlowEntryName",
        "EventFlowEntryPoint",
        "EntryPointName",
    ]
    .into_iter()
    .find_map(|key| parameters.get(key).and_then(|v| v.as_string().ok()))
    .filter(|s| !s.trim().is_empty())
    .map(ToString::to_string);
    let mut references: Vec<_> = parameters
        .iter()
        .filter_map(|(key, value)| {
            let kind = parameter_kind(key)?;
            if matches!(value, Byml::Null | Byml::Map(_) | Byml::Array(_)) {
                return None;
            }
            let text = super::properties::value_text(value);
            if text.trim().is_empty() {
                return None;
            }
            Some(SystemReference {
                source: reference,
                kind,
                parameter: key.to_string(),
                value: text,
                entry_point: (kind == LinkTargetKind::Event)
                    .then(|| entry_point.clone())
                    .flatten(),
            })
        })
        .collect();
    references.sort_by(|a, b| (a.kind.index(), &a.parameter).cmp(&(b.kind.index(), &b.parameter)));
    references
}

fn actor_semantic_kind(
    actor: &Byml,
    visual: Option<&super::ActorVisual>,
) -> (LinkTargetKind, LinkKindSource) {
    let name = uking_library::map::actor_name(actor);
    let profile = visual.and_then(|v| semantic_kind(&v.profile)).or_else(|| {
        actor
            .as_map()
            .ok()
            .and_then(|map| map.get("Profile"))
            .and_then(|value| value.as_string().ok())
            .and_then(|profile| semantic_kind(profile))
    });
    if let Some(kind) = profile.filter(|k| *k != LinkTargetKind::External) {
        return (kind, LinkKindSource::Profile);
    }
    if visual.is_none_or(|v| !v.has_model) {
        if let Some(kind) = semantic_kind(name) {
            return (kind, LinkKindSource::ActorName);
        }
        if !name.starts_with("LinkTag") {
            let references = parameter_references(ActorRef::Current(0), actor);
            if let Some(first) = references.first().filter(|first| {
                references
                    .iter()
                    .all(|reference| reference.kind == first.kind)
            }) {
                return (first.kind, LinkKindSource::Parameters);
            }
        }
    }
    profile
        .map(|kind| (kind, LinkKindSource::Profile))
        .unwrap_or((LinkTargetKind::MapObject, LinkKindSource::MapObject))
}

impl MapDocumentState {
    pub fn has_context_reference(&self, id: u32) -> bool {
        self.assets.context_actors.iter().any(|a| {
            a.as_map()
                .ok()
                .and_then(|m| m.get("LinksToObj"))
                .and_then(|v| v.as_array().ok())
                .is_some_and(|links| links.iter().any(|link| link_destination(link) == Some(id)))
        })
    }
    pub fn actor(&self, reference: ActorRef) -> Option<&Byml> {
        match reference {
            ActorRef::Current(i) => self.file.actors().get(i),
            ActorRef::Context(i) => self.assets.context_actors.get(i),
        }
    }
    pub fn selected_ref(&self) -> Option<ActorRef> {
        self.selected
            .map(ActorRef::Current)
            .or_else(|| self.selected_context.map(ActorRef::Context))
    }
    pub fn selected_actor(&self) -> Option<&Byml> {
        self.actor(self.selected_ref()?)
    }

    pub fn classify_link_target(
        &self,
        target: Option<ActorRef>,
        definition: &str,
    ) -> (LinkTargetKind, LinkKindSource) {
        if let Some(reference) = target {
            if let Some(actor) = self.actor(reference) {
                let visual = self
                    .assets
                    .actors
                    .get(uking_library::map::actor_name(actor));
                return actor_semantic_kind(actor, visual);
            }
        }
        semantic_kind(definition)
            .map(|kind| (kind, LinkKindSource::Definition))
            .unwrap_or((LinkTargetKind::Unknown, LinkKindSource::Unknown))
    }
    pub fn select_ref(&mut self, reference: Option<ActorRef>) {
        self.selected = None;
        self.selected_context = None;
        self.input = None;
        self.input_error = None;
        match reference {
            Some(ActorRef::Current(i)) if i < self.file.actors().len() => self.selected = Some(i),
            Some(ActorRef::Context(i)) if i < self.assets.context_actors.len() => {
                self.selected_context = Some(i)
            }
            _ => (),
        }
        self.rebuild_instances();
    }
    pub fn resolve_hash(&self, hash: u32) -> Option<ActorRef> {
        self.file
            .actors()
            .iter()
            .position(|a| hash_id(a) == Some(hash))
            .map(ActorRef::Current)
            .or_else(|| {
                self.assets
                    .context_actors
                    .iter()
                    .position(|a| hash_id(a) == Some(hash))
                    .map(ActorRef::Context)
            })
    }
    pub fn toggle_object_visibility(&mut self, reference: ActorRef) {
        let hidden = Arc::make_mut(&mut self.hidden_objects);
        if !hidden.remove(&reference) {
            hidden.insert(reference);
        }
        self.rebuild_instances();
    }
    pub fn object_links(
        &self,
        reference: ActorRef,
        incoming: bool,
    ) -> impl ExactSizeIterator<Item = &ObjectLink> {
        let lookup = if incoming {
            &self.link_incoming
        } else {
            &self.link_outgoing
        };
        lookup
            .get(&reference)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(|&index| &self.links[index])
    }

    pub fn object_system_references(
        &self,
        reference: ActorRef,
    ) -> impl ExactSizeIterator<Item = &SystemReference> {
        self.reference_indices
            .get(&reference)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(|&index| &self.system_references[index])
    }

    pub fn rebuild_links(&mut self) {
        // Selection and display switches do not change BYML or ActorInfo.
        // Reuse semantic analysis and only refresh the one-hop neighborhood.
        if self
            .link_cache_source
            .as_ref()
            .is_some_and(|(root, assets)| {
                Arc::ptr_eq(root, &self.file.root) && Arc::ptr_eq(assets, &self.assets)
            })
        {
            self.rebuild_link_context();
            return;
        }
        let actors: Vec<_> = self
            .file
            .actors()
            .iter()
            .enumerate()
            .map(|(i, a)| (ActorRef::Current(i), a))
            .chain(
                self.assets
                    .context_actors
                    .iter()
                    .enumerate()
                    .map(|(i, a)| (ActorRef::Context(i), a)),
            )
            .collect();
        let lookup: HashMap<_, _> = actors
            .iter()
            .rev()
            .filter_map(|(r, a)| hash_id(a).map(|id| (id, *r)))
            .collect();
        let mut result = Vec::new();
        let mut kinds = HashMap::new();
        let mut references = Vec::new();
        for &(source, actor) in &actors {
            kinds.insert(source, self.classify_link_target(Some(source), "").0);
            references.extend(parameter_references(source, actor));
            let Some(Byml::Array(links)) = actor.as_map().ok().and_then(|m| m.get("LinksToObj"))
            else {
                continue;
            };
            for (index, link) in links.iter().enumerate() {
                let Some(destination) = link_destination(link) else {
                    continue;
                };
                let target = lookup.get(&destination).copied();
                let definition = link
                    .as_map()
                    .ok()
                    .and_then(|m| m.get("DefinitionName"))
                    .and_then(|v| v.as_string().ok())
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let (target_kind, kind_source) = self.classify_link_target(target, &definition);
                result.push(ObjectLink {
                    source,
                    target,
                    target_kind,
                    kind_source,
                    index,
                    destination,
                    definition,
                    from: Transform::from_actor(actor).translation,
                    to: target
                        .and_then(|r| self.actor(r))
                        .map(|a| Transform::from_actor(a).translation),
                });
            }
        }
        let mut related_kinds = HashMap::new();
        let mut incoming: HashMap<_, Vec<_>> = HashMap::new();
        let mut outgoing: HashMap<_, Vec<_>> = HashMap::new();
        let mut reference_indices: HashMap<_, Vec<_>> = HashMap::new();
        for (index, link) in result.iter().enumerate() {
            *related_kinds.entry(link.source).or_insert(0u8) |= 1 << link.target_kind.index();
            outgoing.entry(link.source).or_default().push(index);
            if let Some(target) = link.target {
                incoming.entry(target).or_default().push(index);
            }
        }
        for (index, reference) in references.iter().enumerate() {
            *related_kinds.entry(reference.source).or_insert(0u8) |= 1 << reference.kind.index();
            reference_indices
                .entry(reference.source)
                .or_default()
                .push(index);
        }
        self.link_incoming = Arc::new(incoming);
        self.link_outgoing = Arc::new(outgoing);
        self.reference_indices = Arc::new(reference_indices);
        self.actor_kinds = Arc::new(kinds);
        self.related_kinds = Arc::new(related_kinds);
        self.system_references = Arc::new(references);
        self.links = Arc::new(result);
        self.link_cache_source = Some((self.file.root.clone(), self.assets.clone()));
        self.rebuild_link_context();
    }

    fn rebuild_link_context(&mut self) {
        let mut connected = HashSet::new();
        let mut indices = Vec::new();
        if let Some(selected) = self.selected_ref() {
            connected.insert(selected);
            indices.extend(
                self.link_incoming
                    .get(&selected)
                    .into_iter()
                    .flatten()
                    .copied(),
            );
            indices.extend(
                self.link_outgoing
                    .get(&selected)
                    .into_iter()
                    .flatten()
                    .copied(),
            );
            indices.sort_unstable();
            indices.dedup();
            for &index in &indices {
                let link = &self.links[index];
                connected.insert(link.source);
                connected.extend(link.target);
            }
        }
        self.selected_link_indices = Arc::new(indices);
        self.context_objects = Arc::new(connected);
    }
}

#[cfg(test)]
mod tests {
    use super::{parameter_references, semantic_kind, ActorRef, LinkKindSource, LinkTargetKind};
    use crate::model::map_editor::{ActorVisual, MapDocumentState};
    use roead::byml::Byml;
    use std::sync::Arc;
    use uking_library::map::MapFile;

    #[test]
    fn semantic_link_targets_are_classified_without_a_resolved_actor() {
        assert_eq!(
            semantic_kind("CameraEvent_03"),
            Some(LinkTargetKind::Camera)
        );
        assert_eq!(semantic_kind("EventFlowStart"), Some(LinkTargetKind::Event));
        assert_eq!(
            semantic_kind("RainWeatherPreset"),
            Some(LinkTargetKind::Weather)
        );
        assert_eq!(semantic_kind("BasicSig"), None);
        for name in [
            "Terrain",
            "Window",
            "Enemy_Thunder_Weapon",
            "LinkTagAnd",
            "LinkTagOr",
        ] {
            assert_eq!(semantic_kind(name), None, "{name} is not a subsystem");
        }
    }

    #[test]
    fn classify_link_target_prefers_resolved_actor_then_definition() {
        let root = Byml::from_text(
            "Objs: [{UnitConfigName: CameraActor, HashId: !u 1}, {UnitConfigName: PlainActor, HashId: !u 2}]",
        )
        .unwrap();
        let file = MapFile::from_bytes(&root.to_binary(roead::Endian::Little)).unwrap();
        let mut doc = MapDocumentState::empty(file, "links-test", "links-test");
        for name in ["CameraActor", "PlainActor"] {
            Arc::make_mut(&mut doc.assets).actors.insert(
                name.into(),
                ActorVisual {
                    meshes: Vec::new(),
                    resource: String::new(),
                    profile: String::new(),
                    has_model: false,
                    diagnostic: None,
                },
            );
        }
        assert_eq!(
            doc.classify_link_target(Some(ActorRef::Current(0)), "BasicSig"),
            (LinkTargetKind::Camera, LinkKindSource::ActorName)
        );
        assert_eq!(
            doc.classify_link_target(Some(ActorRef::Current(1)), "BasicSig"),
            (LinkTargetKind::MapObject, LinkKindSource::MapObject)
        );
        assert_eq!(
            doc.classify_link_target(None, "WeatherPreset"),
            (LinkTargetKind::Weather, LinkKindSource::Definition)
        );
        assert_eq!(
            doc.classify_link_target(None, "BasicSig"),
            (LinkTargetKind::Unknown, LinkKindSource::Unknown)
        );
    }

    #[test]
    fn resolved_subsystems_retain_world_endpoints_and_connectors_keep_their_kind() {
        let root = Byml::from_text(
            r#"
Objs:
  - UnitConfigName: LinkTagAnd
    HashId: !u 1
    Translate: [1.0, 2.0, 3.0]
    LinksToObj:
      - {DestUnitHashId: !u 2, DefinitionName: BasicSig}
      - {DestUnitHashId: !u 3, DefinitionName: BasicSig}
      - {DestUnitHashId: !u 4, DefinitionName: BasicSig}
      - {DestUnitHashId: !u 999, DefinitionName: BasicSig}
  - {UnitConfigName: CameraTag, HashId: !u 2, Translate: [4.0, 5.0, 6.0]}
  - {UnitConfigName: EventTag, HashId: !u 3, Translate: [7.0, 8.0, 9.0]}
  - {UnitConfigName: WeatherTag, HashId: !u 4, Translate: [10.0, 11.0, 12.0]}
"#,
        )
        .unwrap();
        let file = MapFile::from_bytes(&root.to_binary(roead::Endian::Little)).unwrap();
        let mut doc = MapDocumentState::empty(file, "links-test", "links-test");
        doc.rebuild_links();
        assert_eq!(
            doc.actor_kind(ActorRef::Current(0)),
            LinkTargetKind::MapObject
        );
        for (index, kind, position) in [
            (0, LinkTargetKind::Camera, [4., 5., 6.]),
            (1, LinkTargetKind::Event, [7., 8., 9.]),
            (2, LinkTargetKind::Weather, [10., 11., 12.]),
        ] {
            let link = &doc.links[index];
            assert_eq!(link.target_kind, kind);
            assert_eq!(link.target, Some(ActorRef::Current(index + 1)));
            assert_eq!(link.to, Some(position));
        }
        let unresolved = &doc.links[3];
        assert_eq!(unresolved.target_kind, LinkTargetKind::Unknown);
        assert_eq!(unresolved.kind_source, LinkKindSource::Unknown);
        assert_eq!(unresolved.to, None);
    }

    #[test]
    fn concrete_system_parameters_are_separate_from_hash_links() {
        let actor = Byml::from_text(
            r#"
UnitConfigName: LinkTagAnd
HashId: !u 1
"!Parameters":
  EventFlowName: Demo001
  EventFlowEntryName: Start
  CameraPresetName: CloseUp
  WeatherName: ""
  WeatherPreset: "   "
  SoundName: null
  ArbitraryEventField: unrelated
LinksToObj: [{DestUnitHashId: !u 999, DefinitionName: BasicSig}]
"#,
        )
        .unwrap();
        let references = parameter_references(ActorRef::Current(0), &actor);
        assert_eq!(references.len(), 2);
        let event = references
            .iter()
            .find(|r| r.kind == LinkTargetKind::Event)
            .unwrap();
        assert_eq!(event.value, "Demo001");
        assert_eq!(event.entry_point.as_deref(), Some("Start"));
        let camera = references
            .iter()
            .find(|r| r.kind == LinkTargetKind::Camera)
            .unwrap();
        assert_eq!(camera.value, "CloseUp");
        assert_eq!(camera.entry_point, None);
        assert_eq!(
            super::actor_semantic_kind(&actor, None),
            (LinkTargetKind::MapObject, LinkKindSource::MapObject)
        );
    }

    #[test]
    fn empty_parameters_do_not_classify_actor_as_a_subsystem() {
        let actor =
            Byml::from_text(r#"{UnitConfigName: LinkTagAnd, "!Parameters": {CameraName: ""}}"#)
                .unwrap();
        assert_eq!(
            super::actor_semantic_kind(&actor, None),
            (LinkTargetKind::MapObject, LinkKindSource::MapObject)
        );
    }

    #[test]
    fn mixed_references_do_not_arbitrarily_assign_one_subsystem() {
        let actor = Byml::from_text(r#"{UnitConfigName: ProbeTag, "!Parameters": {CameraName: CloseUp, EventFlowName: Demo001}}"#).unwrap();
        assert_eq!(
            super::actor_semantic_kind(&actor, None),
            (LinkTargetKind::MapObject, LinkKindSource::MapObject)
        );
    }

    #[test]
    fn indexed_links_cover_self_paired_and_unresolved_relationships_once() {
        let root = Byml::from_text(
            r#"
Objs:
  - UnitConfigName: LinkTagAnd
    HashId: !u 1
    "!Parameters": {EventFlowName: Demo001, CameraName: CloseUp}
    LinksToObj:
      - {DestUnitHashId: !u 1, DefinitionName: BasicSig}
      - {DestUnitHashId: !u 2, DefinitionName: BasicSig}
      - {DestUnitHashId: !u 999, DefinitionName: BasicSig}
  - UnitConfigName: WeatherTag
    HashId: !u 3
    LinksToObj: [{DestUnitHashId: !u 1, DefinitionName: BasicSig}]
"#,
        )
        .unwrap();
        let file = MapFile::from_bytes(&root.to_binary(roead::Endian::Little)).unwrap();
        let mut doc = MapDocumentState::empty(file, "index-test", "index-test");
        Arc::make_mut(&mut doc.assets).context_actors.push(Byml::from_text(
            "{UnitConfigName: CameraTag, HashId: !u 2, LinksToObj: [{DestUnitHashId: !u 1, DefinitionName: BasicSig}]}"
        ).unwrap());
        doc.select_ref(Some(ActorRef::Current(0)));
        assert_eq!(doc.object_links(ActorRef::Current(0), false).len(), 3);
        assert_eq!(doc.object_links(ActorRef::Current(0), true).len(), 3);
        assert_eq!(doc.object_system_references(ActorRef::Current(0)).len(), 2);
        assert_eq!(doc.selected_link_indices.as_slice(), &[0, 1, 2, 3, 4]);
        doc.select_ref(Some(ActorRef::Context(0)));
        assert_eq!(doc.selected_link_indices.as_slice(), &[1, 4]);
        doc.select_ref(None);
        assert!(doc.selected_link_indices.is_empty());
        assert!(doc.context_objects.is_empty());
    }
}
