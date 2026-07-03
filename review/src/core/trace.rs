use crate::core::state::OpenContradiction;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MutationTarget {
    M,
    V,
    A,
    B,
    U,
    W,
    Generation,
    Label,
    Allocation,
    Goal,
    Alias,
    Free,
    Contradiction,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cause {
    Allocate,
    Cue,
    AssertActivate,
    Settle,
    Write,
    VarianceUpdate,
    Hebb,
    LinkHint,
    LinkDecay,
    LinkPrune,
    ContradictionOpen,
    ContradictionClose,
    ActivationDecay,
    BaselineDecay,
    GarbageCollect,
    Merge,
    GoalPush,
    GoalPop,
    FreeRow,
    RowGeneration,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MutationRecord {
    pub tick: i64,
    pub event_id: i64,
    pub target: MutationTarget,
    pub row: usize,
    pub axis: Option<usize>,
    pub target_row: Option<usize>,
    pub before: f32,
    pub after: f32,
    pub delta: f32,
    pub cause: Cause,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StepTrace {
    pub tick: i64,
    pub event_id: i64,
    pub active_set_before_decay: Vec<usize>,
    pub settle_iters: usize,
    pub mutations: Vec<MutationRecord>,
    pub opened_contradictions: Vec<OpenContradiction>,
    pub closed_contradictions: Vec<OpenContradiction>,
}

impl StepTrace {
    pub fn new(tick: i64, event_id: i64) -> Self {
        Self {
            tick,
            event_id,
            active_set_before_decay: Vec::new(),
            settle_iters: 0,
            mutations: Vec::new(),
            opened_contradictions: Vec::new(),
            closed_contradictions: Vec::new(),
        }
    }
}
