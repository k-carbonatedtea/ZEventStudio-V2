use super::MapDocumentState;
use roead::byml::Byml;
use std::{collections::HashMap, sync::Arc};
use uking_library::map::{hash_id, Transform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorRef {
    Current(usize),
    Context(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectLink {
    pub source: ActorRef,
    pub target: Option<ActorRef>,
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
    pub fn object_visible(&self, reference: ActorRef) -> bool {
        if self.hidden_objects.contains(&reference) {
            return false;
        }
        if matches!(reference, ActorRef::Context(_)) && !self.show_context {
            return false;
        }
        self.actor(reference)
            .and_then(|a| self.assets.actors.get(uking_library::map::actor_name(a)))
            .is_some_and(|v| {
                if v.has_model {
                    self.show_models
                } else {
                    self.show_tags || self.selected_ref() == Some(reference)
                }
            })
    }
    pub fn toggle_object_visibility(&mut self, reference: ActorRef) {
        let hidden = Arc::make_mut(&mut self.hidden_objects);
        if !hidden.remove(&reference) {
            hidden.insert(reference);
        }
        self.rebuild_instances();
    }
    pub fn rebuild_links(&mut self) {
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
        for (source, actor) in actors {
            let Some(Byml::Array(links)) = actor.as_map().ok().and_then(|m| m.get("LinksToObj"))
            else {
                continue;
            };
            for (index, link) in links.iter().enumerate() {
                let Some(destination) = link_destination(link) else {
                    continue;
                };
                let target = lookup.get(&destination).copied();
                result.push(ObjectLink {
                    source,
                    target,
                    index,
                    destination,
                    definition: link
                        .as_map()
                        .ok()
                        .and_then(|m| m.get("DefinitionName"))
                        .and_then(|v| v.as_string().ok())
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    from: Transform::from_actor(actor).translation,
                    to: target
                        .and_then(|r| self.actor(r))
                        .map(|a| Transform::from_actor(a).translation),
                });
            }
        }
        self.links = Arc::new(result);
    }
}
