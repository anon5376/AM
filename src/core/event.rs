use crate::core::axes::axis_index;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum ConceptRef {
    Id(usize),
    Label(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cue {
    pub concept: ConceptRef,
    pub strength: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Assert {
    pub concept: ConceptRef,
    pub targets: BTreeMap<String, f32>,
    pub weight: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LinkAssert {
    pub left: ConceptRef,
    pub right: ConceptRef,
    pub hint: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", content = "concept")]
pub enum GoalOp {
    Push(ConceptRef),
    Pop(ConceptRef),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: i64,
    pub cues: Vec<Cue>,
    pub asserts: Vec<Assert>,
    pub links: Vec<LinkAssert>,
    pub goal_ops: Vec<GoalOp>,
}

impl Event {
    pub fn empty(id: i64) -> Self {
        Self {
            id,
            cues: Vec::new(),
            asserts: Vec::new(),
            links: Vec::new(),
            goal_ops: Vec::new(),
        }
    }

    pub fn validate_and_clamp(&mut self) -> Result<()> {
        for cue in &self.cues {
            validate_ref(&cue.concept)?;
            anyhow::ensure!(
                cue.strength.is_finite() && cue.strength > 0.0 && cue.strength <= 1.0,
                "cue strength must be in (0,1]"
            );
        }
        for assert in &mut self.asserts {
            validate_ref(&assert.concept)?;
            anyhow::ensure!(
                assert.weight.is_finite() && assert.weight >= 0.0 && assert.weight <= 1.0,
                "assert weight must be in [0,1]"
            );
            for (axis, value) in &mut assert.targets {
                axis_index(axis).with_context(|| format!("unknown axis {axis}"))?;
                anyhow::ensure!(value.is_finite(), "axis value must be finite");
                *value = value.clamp(-2.0, 2.0);
            }
        }
        for link in &self.links {
            validate_ref(&link.left)?;
            validate_ref(&link.right)?;
            anyhow::ensure!(
                link.hint.is_finite() && link.hint >= -1.0 && link.hint <= 1.0,
                "link hint must be in [-1,1]"
            );
        }
        for op in &self.goal_ops {
            match op {
                GoalOp::Push(concept) | GoalOp::Pop(concept) => validate_ref(concept)?,
            }
        }
        Ok(())
    }
}

fn validate_ref(reference: &ConceptRef) -> Result<()> {
    match reference {
        ConceptRef::Id(_) => Ok(()),
        ConceptRef::Label(label) => {
            anyhow::ensure!(!label.trim().is_empty(), "label cannot be empty");
            Ok(())
        }
    }
}
