use super::{MapDocumentState, MapInput};
use crate::model::model_preview::ModelDocumentState;
use roead::byml::Byml;
use std::sync::Arc;
use uking_library::map::{actor_name, hash_id};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyPart {
    Key(String),
    Index(usize),
}
pub type PropertyPath = Vec<PropertyPart>;
pub fn key(name: &str) -> PropertyPart {
    PropertyPart::Key(name.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Float,
    Int,
    Uint,
    String,
    Bool,
    Double,
    Long,
    Ulong,
    Map,
    Array,
    Null,
}
impl PropertyType {
    pub const ALL: [Self; 11] = [
        Self::Float,
        Self::Int,
        Self::Uint,
        Self::String,
        Self::Bool,
        Self::Double,
        Self::Long,
        Self::Ulong,
        Self::Map,
        Self::Array,
        Self::Null,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Float => "Float",
            Self::Int => "Int",
            Self::Uint => "Uint",
            Self::String => "String",
            Self::Bool => "Bool",
            Self::Double => "Double",
            Self::Long => "Long",
            Self::Ulong => "ULong",
            Self::Map => "Map",
            Self::Array => "Array",
            Self::Null => "Null",
        }
    }
    pub fn value(self) -> Byml {
        match self {
            Self::Float => Byml::Float(0.),
            Self::Int => Byml::I32(0),
            Self::Uint => Byml::U32(0),
            Self::String => Byml::String(String::new().into()),
            Self::Bool => Byml::Bool(false),
            Self::Double => Byml::Double(0.),
            Self::Long => Byml::I64(0),
            Self::Ulong => Byml::U64(0),
            Self::Map => Byml::Map(Default::default()),
            Self::Array => Byml::Array(vec![]),
            Self::Null => Byml::Null,
        }
    }
}
pub fn value_at<'a>(mut value: &'a Byml, path: &[PropertyPart]) -> Option<&'a Byml> {
    for part in path {
        value = match part {
            PropertyPart::Key(k) => value.as_map().ok()?.get(k.as_str())?,
            PropertyPart::Index(i) => value.as_array().ok()?.get(*i)?,
        };
    }
    Some(value)
}
fn value_at_mut<'a>(mut value: &'a mut Byml, path: &[PropertyPart]) -> Option<&'a mut Byml> {
    for part in path {
        value = match part {
            PropertyPart::Key(k) => value.as_mut_map().ok()?.get_mut(k.as_str())?,
            PropertyPart::Index(i) => value.as_mut_array().ok()?.get_mut(*i)?,
        };
    }
    Some(value)
}
pub fn value_text(value: &Byml) -> String {
    match value {
        Byml::String(v) => v.to_string(),
        Byml::Bool(v) => v.to_string(),
        Byml::I32(v) => v.to_string(),
        Byml::U32(v) => v.to_string(),
        Byml::I64(v) => v.to_string(),
        Byml::U64(v) => v.to_string(),
        Byml::Float(v) => v.to_string(),
        Byml::Double(v) => v.to_string(),
        _ => value.to_text(),
    }
}
pub fn type_name(value: &Byml) -> &'static str {
    match value {
        Byml::String(_) => "String",
        Byml::Bool(_) => "Bool",
        Byml::I32(_) => "Int",
        Byml::U32(_) => "Uint",
        Byml::I64(_) => "Long",
        Byml::U64(_) => "ULong",
        Byml::Float(_) => "Float",
        Byml::Double(_) => "Double",
        Byml::Map(_) => "Map",
        Byml::Array(_) => "Array",
        Byml::Null => "Null",
        _ => "BYML",
    }
}
pub fn parse_hash(text: &str) -> Result<u32, String> {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16)
    } else {
        text.parse()
    }
    .map_err(|_| "HashId 必须是 Uint32 十进制或 0x 十六进制".into())
}
fn parse_like(value: &Byml, text: &str) -> Result<Byml, String> {
    let error = || format!("请输入有效的 {} 值", type_name(value));
    let s = text.trim();
    Ok(match value {
        Byml::String(_) => Byml::String(text.into()),
        Byml::Bool(_) => Byml::Bool(s.parse().map_err(|_| error())?),
        Byml::I32(_) => Byml::I32(s.parse().map_err(|_| error())?),
        Byml::U32(_) => Byml::U32(parse_hash(s)?),
        Byml::I64(_) => Byml::I64(s.parse().map_err(|_| error())?),
        Byml::U64(_) => Byml::U64(s.parse().map_err(|_| error())?),
        Byml::Float(_) => {
            let n: f32 = s.parse().map_err(|_| error())?;
            if !n.is_finite() {
                return Err(error());
            }
            Byml::Float(n)
        }
        Byml::Double(_) => {
            let n: f64 = s.parse().map_err(|_| error())?;
            if !n.is_finite() {
                return Err(error());
            }
            Byml::Double(n)
        }
        _ => Byml::from_text(s).map_err(|e| e.to_string())?,
    })
}

impl MapDocumentState {
    /// Commit only a fully validated copy, so failed edits cannot dirty the map
    /// or consume an undo checkpoint. Unknown BYML nodes keep their native types.
    fn edit_actor(
        &mut self,
        edit: impl FnOnce(&mut Byml) -> Result<(), String>,
    ) -> Result<(), String> {
        let index = self
            .selected
            .ok_or("配套地图对象为只读，请先打开其所属地图")?;
        let before = self.file.actors().get(index).ok_or("对象不存在")?;
        let mut actor = before.clone();
        edit(&mut actor)?;
        let props = actor.as_map().map_err(|e| e.to_string())?;
        if actor_name(&actor).trim().is_empty()
            || !matches!(props.get("UnitConfigName"), Some(Byml::String(_)))
        {
            return Err("UnitConfigName 必须是非空字符串".into());
        }
        let id = hash_id(&actor).ok_or("HashId 必须是 Int32 或 Uint32")?;
        if self
            .file
            .actors()
            .iter()
            .enumerate()
            .any(|(i, a)| i != index && hash_id(a) == Some(id))
            || self
                .assets
                .context_actors
                .iter()
                .any(|a| hash_id(a) == Some(id))
        {
            return Err("HashId 与当前或配套地图对象重复".into());
        }
        if let Some(v) = props.get("SRTHash") {
            if !matches!(v, Byml::I32(_)) {
                return Err("SRTHash 必须是 Int32".into());
            }
        }
        if let Some(links) = props.get("LinksToObj") {
            for link in links.as_array().map_err(|_| "LinksToObj 必须是数组")? {
                let m = link.as_map().map_err(|_| "链接必须是 Map")?;
                if !m
                    .get("DefinitionName")
                    .and_then(|v| v.as_string().ok())
                    .is_some_and(|s| !s.trim().is_empty())
                    || super::links::link_destination(link).is_none()
                {
                    return Err("链接需要非空 DefinitionName 和 Uint32 DestUnitHashId".into());
                }
            }
        }
        if &actor == before {
            return Ok(());
        }
        let old_id = hash_id(before);
        let reload = actor_name(before) != actor_name(&actor);
        if old_id != Some(id) && old_id.is_some_and(|id| self.has_context_reference(id)) {
            return Err("配套地图中有链接指向此对象，无法单独修改 HashId".into());
        }
        let mut root = self.file.root.as_ref().clone();
        let actors = root
            .as_mut_map()
            .map_err(|e| e.to_string())?
            .get_mut("Objs")
            .ok_or("缺少 Objs")?
            .as_mut_array()
            .map_err(|e| e.to_string())?;
        actors[index] = actor;
        if old_id != Some(id) {
            for a in actors {
                if let Some(Byml::Array(links)) =
                    a.as_mut_map().ok().and_then(|m| m.get_mut("LinksToObj"))
                {
                    for link in links {
                        if super::links::link_destination(link) == old_id {
                            link.as_mut_map()
                                .unwrap()
                                .insert("DestUnitHashId".into(), Byml::U32(id));
                        }
                    }
                }
            }
        }
        self.checkpoint();
        self.file.root = Arc::new(root);
        if reload {
            self.reload_models();
        } else {
            self.rebuild_instances();
        }
        Ok(())
    }
    pub fn reload_models(&mut self) {
        let camera = self.scene.camera.clone();
        self.scene = ModelDocumentState::from_bfres(
            &self.scene.file_path,
            &self.scene.model_name,
            &Default::default(),
        );
        self.scene.camera = camera;
        self.scene.show_grid = false;
        self.scene.show_axes = false;
        self.scene.is_switch = !self.file.big_endian;
        self.loading = true;
        self.load_started = false;
    }
    pub fn refresh_after_history(&mut self) {
        if !self.assets.actors.is_empty()
            && self
                .file
                .actors()
                .iter()
                .any(|a| !self.assets.actors.contains_key(actor_name(a)))
        {
            self.reload_models();
        } else {
            self.rebuild_instances();
        }
    }
    pub fn set_property(&mut self, path: &[PropertyPart], value: Byml) -> Result<(), String> {
        if path.is_empty() {
            return Err("不能替换整个对象".into());
        }
        // A new/changed destination must resolve. Existing unresolved links are
        // retained when unrelated properties are edited.
        if matches!(path,[PropertyPart::Key(k),PropertyPart::Index(_),PropertyPart::Key(field)] if k=="LinksToObj" && field=="DestUnitHashId")
        {
            let id = match &value {
                Byml::U32(v) => Some(*v),
                Byml::I32(v) => Some(*v as u32),
                _ => None,
            }
            .ok_or("链接目标必须是 Uint32")?;
            if self.resolve_hash(id).is_none() {
                return Err(format!("找不到链接目标 0x{id:08X}"));
            }
        }
        self.edit_actor(|a| {
            *value_at_mut(a, path).ok_or("属性不存在")? = value;
            Ok(())
        })
    }
    pub fn remove_property(&mut self, path: &[PropertyPart]) -> Result<(), String> {
        if path.len() == 1
            && matches!(&path[0],PropertyPart::Key(k) if ["HashId","UnitConfigName","Translate","Rotate","Scale"].contains(&k.as_str()))
        {
            return Err("基础对象字段不能删除".into());
        }
        let (last, parent) = path.split_last().ok_or("请选择属性")?;
        self.edit_actor(|a| {
            let parent = value_at_mut(a, parent).ok_or("属性不存在")?;
            match last {
                PropertyPart::Key(k) => {
                    parent
                        .as_mut_map()
                        .map_err(|e| e.to_string())?
                        .remove(k.as_str());
                }
                PropertyPart::Index(i) => {
                    let array = parent.as_mut_array().map_err(|e| e.to_string())?;
                    if *i >= array.len() {
                        return Err("数组项不存在".into());
                    }
                    array.remove(*i);
                }
            }
            Ok(())
        })
    }
    pub fn add_property(
        &mut self,
        parent: &[PropertyPart],
        name: &str,
        kind: PropertyType,
    ) -> Result<(), String> {
        self.edit_actor(|a| {
            let parent = value_at_mut(a, parent).ok_or("父属性不存在")?;
            match parent {
                Byml::Map(m) => {
                    if name.trim().is_empty() {
                        return Err("属性名不能为空".into());
                    }
                    if m.contains_key(name) {
                        return Err("属性名已存在".into());
                    }
                    m.insert(name.into(), kind.value());
                }
                Byml::Array(array) => array.push(kind.value()),
                _ => return Err("只能向 Map 或 Array 添加属性".into()),
            }
            Ok(())
        })
    }
    pub fn rename_property(&mut self, path: &[PropertyPart], name: &str) -> Result<(), String> {
        let (PropertyPart::Key(old), parent) = path.split_last().ok_or("请选择属性")? else {
            return Err("数组索引不能重命名".into());
        };
        if parent.is_empty()
            && [
                "HashId",
                "UnitConfigName",
                "SRTHash",
                "Translate",
                "Rotate",
                "Scale",
                "LinksToObj",
                "!Parameters",
            ]
            .contains(&old.as_str())
        {
            return Err("基础字段不能重命名".into());
        }
        self.edit_actor(|a| {
            let m = value_at_mut(a, parent)
                .ok_or("属性不存在")?
                .as_mut_map()
                .map_err(|e| e.to_string())?;
            if name.trim().is_empty() || (name != old && m.contains_key(name)) {
                return Err("属性名为空或已存在".into());
            }
            let v = m.remove(old.as_str()).ok_or("属性不存在")?;
            m.insert(name.into(), v);
            Ok(())
        })
    }
    pub fn begin_property(&mut self, path: PropertyPath) {
        if let Some(value) = self.selected_actor().and_then(|a| value_at(a, &path)) {
            self.input_text = value_text(value);
            self.edit_path = path;
            self.input = Some(MapInput::Property);
            self.input_error = None;
        }
    }
    pub fn apply_property_input(&mut self) -> Result<bool, String> {
        match self.input {
            Some(MapInput::Property) => {
                let path = self.edit_path.clone();
                let value = self
                    .selected_actor()
                    .and_then(|a| value_at(a, &path))
                    .ok_or("属性不存在")?;
                let parsed = parse_like(value, &self.input_text)?;
                self.set_property(&path, parsed)?;
            }
            Some(MapInput::PropertyName) => {
                let path = self.edit_path.clone();
                let name = self.input_text.clone();
                self.rename_property(&path, &name)?;
            }
            Some(MapInput::NewPropertyName) => self.new_property_name = self.input_text.clone(),
            Some(MapInput::LinkDefinition) => self.new_link_definition = self.input_text.clone(),
            Some(MapInput::LinkTarget) => self.new_link_target = self.input_text.clone(),
            _ => return Ok(false),
        }
        Ok(true)
    }
    pub fn add_link(&mut self) -> Result<(), String> {
        let destination = parse_hash(&self.new_link_target)?;
        if self.resolve_hash(destination).is_none() {
            return Err(format!("找不到链接目标 0x{destination:08X}"));
        }
        let definition = self.new_link_definition.trim().to_string();
        if definition.is_empty() {
            return Err("链接类型不能为空".into());
        }
        self.edit_actor(|a| {
            let m = a.as_mut_map().map_err(|e| e.to_string())?;
            let links = m
                .entry("LinksToObj".into())
                .or_insert_with(|| Byml::Array(vec![]))
                .as_mut_array()
                .map_err(|e| e.to_string())?;
            if links.iter().any(|l| {
                super::links::link_destination(l) == Some(destination)
                    && l.as_map()
                        .ok()
                        .and_then(|m| m.get("DefinitionName"))
                        .and_then(|v| v.as_string().ok())
                        .map(|s| s.as_str())
                        == Some(definition.as_str())
            }) {
                return Err("相同类型和目标的链接已存在".into());
            }
            links.push(Byml::Map(
                [
                    ("DefinitionName".into(), Byml::String(definition.into())),
                    ("DestUnitHashId".into(), Byml::U32(destination)),
                    ("!Parameters".into(), Byml::Map(Default::default())),
                ]
                .into_iter()
                .collect(),
            ));
            Ok(())
        })
    }
    pub fn replace_parameters(&mut self, value: Byml) -> Result<(), String> {
        if !matches!(value, Byml::Map(_)) {
            return Err("参数必须是 BYML Map".into());
        }
        self.edit_actor(|a| {
            a.as_mut_map()
                .map_err(|e| e.to_string())?
                .insert("!Parameters".into(), value);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::map_editor::{geometry, loader, ActorRef, ActorVisual};
    use uking_library::map::MapFile;

    fn document() -> MapDocumentState {
        let root=Byml::from_text("Objs:\n- UnitConfigName: Source\n  HashId: !u 1\n  SRTHash: 27\n  '!Parameters': {Enabled: true, Count: 7, Rate: 1.25}\n  LinksToObj:\n  - {DefinitionName: BasicSig, DestUnitHashId: !u 2, '!Parameters': {Unknown: 9}}\n- {UnitConfigName: Target, HashId: !u 2}\nRails: [{Unknown: preserved}]\nUnknownRoot: 17").unwrap();
        let file = MapFile::from_bytes(&root.to_binary(roead::Endian::Little)).unwrap();
        let mut doc = MapDocumentState::empty(file, "test.smubin", "test");
        doc.loading = false;
        doc.selected = Some(0);
        doc.rebuild_instances();
        doc
    }
    #[test]
    fn typed_property_edits_preserve_unknown_data_and_undo_save_roundtrip() {
        let mut doc = document();
        let original = doc.file.root.clone();
        doc.begin_property(vec![key("!Parameters"), key("Count")]);
        doc.input_text = "42".into();
        doc.apply_input().unwrap();
        assert_eq!(
            value_at(
                doc.selected_actor().unwrap(),
                &[key("!Parameters"), key("Count")]
            ),
            Some(&Byml::I32(42))
        );
        assert_eq!(
            value_at(doc.selected_actor().unwrap(), &[key("SRTHash")]),
            Some(&Byml::I32(27))
        );
        assert_eq!(
            doc.file.root.as_map().unwrap()["Rails"],
            original.as_map().unwrap()["Rails"]
        );
        let saved = doc.file.root.clone();
        assert_eq!(
            MapFile::from_bytes(&doc.file.to_bytes()).unwrap().root,
            saved
        );
        doc.undo();
        assert_eq!(doc.file.root, original);
        doc.redo();
        assert_eq!(doc.file.root, saved);
    }
    #[test]
    fn invalid_property_edit_keeps_draft_document_and_undo_unchanged() {
        let mut doc = document();
        let original = doc.file.root.clone();
        doc.begin_property(vec![key("!Parameters"), key("Count")]);
        doc.input_text = "2147483648".into();
        assert!(doc.apply_input().is_err());
        assert_eq!(doc.file.root, original);
        assert!(doc.undo.is_empty());
        assert_eq!(doc.input, Some(MapInput::Property));
        assert!(doc.input_error.is_some());
        assert!(parse_like(&Byml::Float(0.), "NaN").is_err());
        assert!(parse_like(&Byml::Double(0.), "inf").is_err());
        for v in [
            Byml::U32(u32::MAX),
            Byml::I64(i64::MIN),
            Byml::U64(u64::MAX),
            Byml::Double(1.23456789012345),
            Byml::String("中文参数".into()),
        ] {
            assert_eq!(parse_like(&v, &value_text(&v)).unwrap(), v);
        }
    }
    #[test]
    fn hash_changes_retarget_local_links_and_reject_duplicates_atomically() {
        let mut doc = document();
        doc.select(Some(1));
        doc.set_property(&[key("HashId")], Byml::U32(3)).unwrap();
        assert_eq!(
            super::super::links::link_destination(
                &doc.file.actors()[0].as_map().unwrap()["LinksToObj"]
                    .as_array()
                    .unwrap()[0]
            ),
            Some(3)
        );
        assert_eq!(doc.links[0].target, Some(ActorRef::Current(1)));
        let saved = doc.file.root.clone();
        let checkpoints = doc.undo.len();
        assert!(doc.set_property(&[key("HashId")], Byml::U32(1)).is_err());
        assert_eq!(doc.file.root, saved);
        assert_eq!(doc.undo.len(), checkpoints);
    }
    #[test]
    fn links_resolve_both_directions_across_context_and_keep_unknown_parameters() {
        let mut doc = document();
        let context=Byml::from_text("{UnitConfigName: Context, HashId: !u 9, LinksToObj: [{DefinitionName: ContextSig, DestUnitHashId: !u 1}]}").unwrap();
        Arc::make_mut(&mut doc.assets).context_actors.push(context);
        doc.rebuild_instances();
        assert!(doc
            .links
            .iter()
            .any(|l| l.source == ActorRef::Context(0) && l.target == Some(ActorRef::Current(0))));
        let original = doc.file.root.clone();
        assert!(doc.set_property(&[key("HashId")], Byml::U32(10)).is_err());
        assert_eq!(doc.file.root, original);
        assert!(doc.undo.is_empty());
        assert!(doc.delete().is_err());
        assert_eq!(doc.file.root, original);
        assert!(doc.undo.is_empty());
        doc.new_link_target = "0x9".into();
        doc.new_link_definition = "BasicSigOnOnly".into();
        doc.add_link().unwrap();
        assert!(doc.add_link().is_err());
        assert!(doc
            .links
            .iter()
            .any(|l| l.source == ActorRef::Current(0) && l.target == Some(ActorRef::Context(0))));
        doc.set_property(
            &[
                key("LinksToObj"),
                PropertyPart::Index(0),
                key("DefinitionName"),
            ],
            Byml::String("NewSig".into()),
        )
        .unwrap();
        assert_eq!(
            value_at(
                &doc.file.actors()[0],
                &[
                    key("LinksToObj"),
                    PropertyPart::Index(0),
                    key("!Parameters"),
                    key("Unknown")
                ]
            ),
            Some(&Byml::I32(9))
        );
        assert!(doc
            .set_property(
                &[
                    key("LinksToObj"),
                    PropertyPart::Index(0),
                    key("DestUnitHashId")
                ],
                Byml::U32(999)
            )
            .is_err());
        doc.select_ref(Some(ActorRef::Context(0)));
        assert!(doc.set_property(&[key("HashId")], Byml::U32(10)).is_err());
    }
    #[test]
    fn nested_properties_add_rename_remove_and_clipboard_keep_types() {
        let mut doc = document();
        let params = vec![key("!Parameters")];
        doc.add_property(&params, "Custom", PropertyType::Array)
            .unwrap();
        let custom = vec![key("!Parameters"), key("Custom")];
        doc.add_property(&custom, "", PropertyType::Ulong).unwrap();
        doc.set_property(
            &[key("!Parameters"), key("Custom"), PropertyPart::Index(0)],
            Byml::U64(u64::MAX),
        )
        .unwrap();
        doc.rename_property(&custom, "Renamed").unwrap();
        let copied = value_at(doc.selected_actor().unwrap(), &params)
            .unwrap()
            .clone();
        let pasted = Byml::from_text(copied.to_text()).unwrap();
        doc.replace_parameters(pasted).unwrap();
        assert_eq!(
            value_at(doc.selected_actor().unwrap(), &params),
            Some(&copied)
        );
        doc.remove_property(&[key("!Parameters"), key("Renamed"), PropertyPart::Index(0)])
            .unwrap();
        assert!(doc.remove_property(&[key("HashId")]).is_err());
        assert!(doc
            .rename_property(&[key("UnitConfigName")], "Other")
            .is_err());
    }
    #[test]
    fn visibility_is_editor_only_and_hidden_objects_are_not_pickable() {
        let mut doc = document();
        let original = doc.file.root.clone();
        let mesh = loader::push_mesh(&mut doc.scene, geometry::cube("test"));
        for (name, has_model) in [("Source", true), ("Target", false)] {
            Arc::make_mut(&mut doc.assets).actors.insert(
                name.into(),
                ActorVisual {
                    meshes: vec![mesh],
                    resource: String::new(),
                    has_model,
                    diagnostic: None,
                },
            );
        }
        doc.select(None);
        doc.show_tags = false;
        doc.rebuild_instances();
        assert_eq!(doc.scene.instances.as_ref().unwrap().len(), 1);
        doc.show_tags = true;
        doc.rebuild_instances();
        assert_eq!(doc.scene.instances.as_ref().unwrap().len(), 2);
        doc.toggle_object_visibility(ActorRef::Current(0));
        assert_eq!(
            doc.instance_owners.as_slice(),
            &[Some(ActorRef::Current(1))]
        );
        doc.show_models = false;
        doc.show_tags = false;
        doc.rebuild_instances();
        assert!(doc.scene.instances.as_ref().unwrap().is_empty());
        assert_eq!(doc.file.root, original);
        assert!(!doc.file.is_modified());
        assert!(doc.undo.is_empty());
    }
    #[test]
    fn actor_type_change_and_history_reload_missing_models() {
        let mut doc = document();
        Arc::make_mut(&mut doc.assets).actors.insert(
            "Source".into(),
            ActorVisual {
                meshes: vec![],
                resource: String::new(),
                has_model: false,
                diagnostic: None,
            },
        );
        doc.set_property(&[key("UnitConfigName")], Byml::String("NewType".into()))
            .unwrap();
        assert!(doc.loading);
        assert!(!doc.load_started);
        doc.loading = false;
        doc.undo();
        assert_eq!(actor_name(&doc.file.actors()[0]), "Source");
        assert!(doc.loading);
    }
}
