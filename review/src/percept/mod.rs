use crate::core::event::{Assert, ConceptRef, Cue, Event, LinkAssert};
use crate::world::observation::{Observation, VisibleEntity};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const F_DIM: usize = 12;
pub const SELF_LABEL: &str = "trk_0";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pose {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub adjacent: bool,
    pub distance_to_self: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Percept {
    pub local_id: u32,
    pub feat: [f32; F_DIM],
    pub pose: Pose,
}

#[derive(Clone, Debug, Default)]
pub struct PerceptBridge {
    seen: BTreeSet<FeatSig>,
}

impl PerceptBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn percepts(&self, observation: &Observation) -> Vec<Percept> {
        let mut entities = observation.visible_entities.clone();
        entities.sort_by_key(|entity| entity.local_id);
        entities.iter().map(percept_from_entity).collect()
    }

    pub fn events_for_observation(&mut self, observation: &Observation) -> Result<Vec<Event>> {
        let mut event = Event::empty(observation.tick as i64);
        let percepts = self.percepts(observation);
        for percept in &percepts {
            let label = local_label(percept.local_id);
            event.cues.push(Cue {
                concept: ConceptRef::Label(label.clone()),
                strength: 0.8,
            });
            let sig = FeatSig::from_feat(&percept.feat);
            let novelty = if self.seen.insert(sig) { 1.0 } else { 0.0 };
            event.asserts.push(Assert {
                concept: ConceptRef::Label(label.clone()),
                targets: entity_targets(novelty),
                weight: 1.0,
            });
            if percept.pose.adjacent {
                event.links.push(LinkAssert {
                    left: ConceptRef::Label(SELF_LABEL.to_string()),
                    right: ConceptRef::Label(label),
                    hint: 0.6,
                });
            }
        }
        event.asserts.push(Assert {
            concept: ConceptRef::Label(SELF_LABEL.to_string()),
            targets: self_targets(observation),
            weight: 1.0,
        });
        event.validate_and_clamp()?;
        Ok(vec![event])
    }
}

pub fn local_label(local_id: u32) -> String {
    format!("loc_{local_id}")
}

fn percept_from_entity(entity: &VisibleEntity) -> Percept {
    Percept {
        local_id: entity.local_id,
        feat: appearance_feat(entity),
        pose: Pose {
            x: entity.x,
            y: entity.y,
            dx: entity.dx,
            dy: entity.dy,
            adjacent: entity.adjacent,
            distance_to_self: entity.distance_to_self,
        },
    }
}

fn appearance_feat(entity: &VisibleEntity) -> [f32; F_DIM] {
    let mut feat = [0.0; F_DIM];
    let shape_slot = ((entity.shape_id.saturating_sub(1)) % 5) as usize;
    let color_slot = (entity.color_id % 6) as usize;
    feat[shape_slot] = 1.0;
    feat[5 + color_slot] = 1.0;
    feat[11] = entity.size as f32 / 3.0;
    feat
}

fn entity_targets(novelty: f32) -> BTreeMap<String, f32> {
    BTreeMap::from([
        ("concreteness".to_string(), 1.0),
        ("novelty".to_string(), novelty),
        ("temporal_near".to_string(), 1.0),
    ])
}

fn self_targets(observation: &Observation) -> BTreeMap<String, f32> {
    let energy = (observation.energy_bucket as f32 / 20.0).clamp(0.0, 1.0);
    let inverse = (1.0 - energy).clamp(0.0, 1.0);
    let mut out = BTreeMap::from([
        ("risk".to_string(), inverse),
        ("temporal_near".to_string(), 1.0),
        ("value".to_string(), energy),
    ]);
    if observation.blocked {
        out.insert("constraint_relevance".to_string(), 1.0);
    }
    out
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FeatSig {
    shape_slot: u8,
    color_slot: u8,
    size_slot: u32,
}

impl FeatSig {
    fn from_feat(feat: &[f32; F_DIM]) -> Self {
        let shape_slot = feat[..5].iter().position(|value| *value > 0.0).unwrap_or(0) as u8;
        let color_slot = feat[5..11]
            .iter()
            .position(|value| *value > 0.0)
            .unwrap_or(0) as u8;
        Self {
            shape_slot,
            color_slot,
            size_slot: feat[11].to_bits(),
        }
    }
}
