use crate::core::event::{ConceptRef, RowRef};
use crate::core::theta::Theta;
use crate::core::trace::{Cause, MutationRecord, MutationTarget, StepTrace};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const SNAPSHOT_FORMAT_VERSION: u32 = 3;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub target: usize,
    pub weight: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WriteEvidence {
    pub event_id: i64,
    pub tick: i64,
    pub sign: i8,
    pub delta: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenContradiction {
    pub concept: RowRef,
    pub axis: usize,
    pub evidence_event_ids: Vec<i64>,
    pub opened_tick: i64,
    pub pressure: f32,
    pub status: ContradictionStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContradictionStatus {
    Open,
    Closed { closed_tick: i64 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AmState {
    pub format_version: u32,
    pub theta: Theta,
    pub tick: i64,
    pub generation: Vec<u32>,
    pub allocated: Vec<bool>,
    pub m: Vec<f32>,
    pub v: Vec<f32>,
    pub a: Vec<f32>,
    pub b: Vec<f32>,
    pub u: Vec<i64>,
    pub links: Vec<Vec<Link>>,
    pub labels: Vec<Option<String>>,
    pub label_to_id: BTreeMap<String, RowRef>,
    pub aliases: BTreeMap<RowRef, RowRef>,
    pub goals: Vec<RowRef>,
    pub open_contradictions: Vec<OpenContradiction>,
    pub recent_writes: BTreeMap<(RowRef, usize), Vec<WriteEvidence>>,
    pub free_list: Vec<usize>,
}

impl AmState {
    pub fn new(theta: Theta) -> Self {
        let n = theta.n;
        let d = theta.d;
        let mut free_list: Vec<usize> = (0..n).collect();
        free_list.reverse();
        Self {
            format_version: SNAPSHOT_FORMAT_VERSION,
            theta,
            tick: 0,
            generation: vec![0; n],
            allocated: vec![false; n],
            m: vec![0.0; n * d],
            v: vec![0.0; n * d],
            a: vec![0.0; n],
            b: vec![0.0; n],
            u: vec![0; n],
            links: vec![Vec::new(); n],
            labels: vec![None; n],
            label_to_id: BTreeMap::new(),
            aliases: BTreeMap::new(),
            goals: Vec::new(),
            open_contradictions: Vec::new(),
            recent_writes: BTreeMap::new(),
            free_list,
        }
    }

    pub fn idx(&self, row: usize, axis: usize) -> usize {
        row * self.theta.d + axis
    }

    pub fn m_get(&self, row: usize, axis: usize) -> f32 {
        self.m[self.idx(row, axis)]
    }

    pub fn v_get(&self, row: usize, axis: usize) -> f32 {
        self.v[self.idx(row, axis)]
    }

    pub fn row_ref(&self, row: usize) -> RowRef {
        RowRef::new(row, self.generation[row])
    }

    pub fn is_allocated(&self, row: usize) -> bool {
        row < self.theta.n && self.allocated[row]
    }

    pub fn resolve_row_ref(&self, reference: RowRef) -> Result<RowRef> {
        let resolved = self.resolve_alias(reference);
        anyhow::ensure!(
            resolved.id < self.theta.n,
            "row ref {}:{} is out of range",
            reference.id,
            reference.gen
        );
        anyhow::ensure!(
            self.generation[resolved.id] == resolved.gen,
            "stale row ref {}:{}; current generation is {}",
            reference.id,
            reference.gen,
            self.generation[resolved.id]
        );
        anyhow::ensure!(
            self.is_allocated(resolved.id),
            "row ref {}:{} is not allocated",
            reference.id,
            reference.gen
        );
        Ok(resolved)
    }

    pub fn allocated_rows_sorted(&self) -> Vec<usize> {
        (0..self.theta.n)
            .filter(|row| self.is_allocated(*row))
            .collect()
    }

    pub fn active_rows_sorted(&self, threshold: f32) -> Vec<usize> {
        self.allocated_rows_sorted()
            .into_iter()
            .filter(|row| self.a[*row] >= threshold)
            .collect()
    }

    pub fn certainty(&self, row: usize) -> f32 {
        let mut sum = 0.0;
        for axis in 0..self.theta.d {
            sum += self.v_get(row, axis);
        }
        1.0 / (1.0 + sum / self.theta.d as f32)
    }

    pub fn snapshot_bytes(&self) -> Vec<u8> {
        crate::core::snapshot::to_bytes(self)
    }

    pub fn state_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.snapshot_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn resolve_alias(&self, mut row: RowRef) -> RowRef {
        let mut seen = BTreeSet::new();
        while let Some(next) = self.aliases.get(&row) {
            if !seen.insert(row) {
                break;
            }
            row = *next;
        }
        row
    }

    pub fn concept_label(&self, row: usize) -> String {
        self.labels
            .get(row)
            .and_then(|label| label.clone())
            .unwrap_or_else(|| format!("#{row}"))
    }

    pub fn resolve_existing(&self, reference: &ConceptRef) -> Result<usize> {
        match reference {
            ConceptRef::Id(row) => {
                let resolved = self.resolve_row_ref(*row)?;
                Ok(resolved.id)
            }
            ConceptRef::Label(label) => {
                let row_ref = self
                    .label_to_id
                    .get(label)
                    .with_context(|| format!("unknown label {label}"))?;
                let resolved = self.resolve_row_ref(*row_ref)?;
                Ok(resolved.id)
            }
        }
    }

    pub fn m_set_logged(
        &mut self,
        row: usize,
        axis: usize,
        after: f32,
        trace: &mut StepTrace,
        cause: Cause,
    ) {
        let idx = self.idx(row, axis);
        let before = self.m[idx];
        if (after - before).abs() < self.theta.eps_log {
            return;
        }
        self.m[idx] = after;
        push_coalesced(
            trace,
            record(
                self.tick,
                trace.event_id,
                MutationTarget::M,
                row,
                Some(axis),
                None,
                before,
                after,
                cause,
            ),
            self.theta.eps_log,
        );
    }

    pub fn v_set_logged(
        &mut self,
        row: usize,
        axis: usize,
        after: f32,
        trace: &mut StepTrace,
        cause: Cause,
    ) {
        let idx = self.idx(row, axis);
        let before = self.v[idx];
        if (after - before).abs() < self.theta.eps_log {
            return;
        }
        self.v[idx] = after;
        push_coalesced(
            trace,
            record(
                self.tick,
                trace.event_id,
                MutationTarget::V,
                row,
                Some(axis),
                None,
                before,
                after,
                cause,
            ),
            self.theta.eps_log,
        );
    }

    pub fn a_set_logged(&mut self, row: usize, after: f32, trace: &mut StepTrace, cause: Cause) {
        let before = self.a[row];
        if (after - before).abs() < self.theta.eps_log {
            return;
        }
        self.a[row] = after.clamp(0.0, 1.0);
        push_coalesced(
            trace,
            record(
                self.tick,
                trace.event_id,
                MutationTarget::A,
                row,
                None,
                None,
                before,
                self.a[row],
                cause,
            ),
            self.theta.eps_log,
        );
    }

    pub fn b_set_logged(&mut self, row: usize, after: f32, trace: &mut StepTrace, cause: Cause) {
        let before = self.b[row];
        if (after - before).abs() < self.theta.eps_log {
            return;
        }
        self.b[row] = after.max(0.0);
        push_coalesced(
            trace,
            record(
                self.tick,
                trace.event_id,
                MutationTarget::B,
                row,
                None,
                None,
                before,
                self.b[row],
                cause,
            ),
            self.theta.eps_log,
        );
    }

    pub fn u_set_logged(&mut self, row: usize, after: i64, trace: &mut StepTrace, cause: Cause) {
        let before = self.u[row];
        if before == after {
            return;
        }
        self.u[row] = after;
        push_coalesced(
            trace,
            record(
                self.tick,
                trace.event_id,
                MutationTarget::U,
                row,
                None,
                None,
                before as f32,
                after as f32,
                cause,
            ),
            self.theta.eps_log,
        );
    }

    pub fn increment_generation_logged(&mut self, row: usize, trace: &mut StepTrace, cause: Cause) {
        let before = self.generation[row];
        let after = before.wrapping_add(1);
        self.generation[row] = after;
        push_coalesced(
            trace,
            record(
                self.tick,
                trace.event_id,
                MutationTarget::Generation,
                row,
                None,
                None,
                before as f32,
                after as f32,
                cause,
            ),
            self.theta.eps_log,
        );
    }

    pub fn allocated_set_logged(
        &mut self,
        row: usize,
        after: bool,
        trace: &mut StepTrace,
        cause: Cause,
    ) {
        let before = self.allocated[row];
        if before == after {
            return;
        }
        self.allocated[row] = after;
        push_coalesced_mutation(
            trace,
            record(
                self.tick,
                trace.event_id,
                MutationTarget::Allocation,
                row,
                None,
                None,
                if before { 1.0 } else { 0.0 },
                if after { 1.0 } else { 0.0 },
                cause,
            ),
            self.theta.eps_log,
        );
    }

    pub fn log_w_change(
        &self,
        trace: &mut StepTrace,
        source: usize,
        target: usize,
        before: f32,
        after: f32,
        cause: Cause,
    ) {
        if (after - before).abs() < self.theta.eps_log {
            return;
        }
        push_coalesced_mutation(
            trace,
            record(
                self.tick,
                trace.event_id,
                MutationTarget::W,
                source,
                None,
                Some(target),
                before,
                after,
                cause,
            ),
            self.theta.eps_log,
        );
    }

    pub fn log_nonscalar(
        &self,
        trace: &mut StepTrace,
        target: MutationTarget,
        row: usize,
        target_row: Option<usize>,
        cause: Cause,
    ) {
        trace.mutations.push(record(
            self.tick,
            trace.event_id,
            target,
            row,
            None,
            target_row,
            0.0,
            1.0,
            cause,
        ));
    }

    pub fn push_write_evidence(
        &mut self,
        row: usize,
        axis: usize,
        event_id: i64,
        sign: i8,
        delta: f32,
    ) {
        let row_ref = self.row_ref(row);
        let history = self.recent_writes.entry((row_ref, axis)).or_default();
        history.push(WriteEvidence {
            event_id,
            tick: self.tick,
            sign,
            delta,
        });
        let keep = self.theta.m_sign_window;
        if history.len() > keep {
            let drop_count = history.len() - keep;
            history.drain(0..drop_count);
        }
    }

    pub fn force_last_touched(&mut self, row: usize, tick: i64) {
        self.u[row] = tick;
    }

    pub fn clear_free_metadata(&mut self, row: usize, old_ref: RowRef, cause: Cause) {
        if let Some(label) = self.labels[row].take() {
            if cause == Cause::Merge {
                self.label_to_id.insert(label, old_ref);
            } else {
                self.label_to_id.remove(&label);
            }
        }
        self.goals.retain(|goal| *goal != old_ref);
        self.recent_writes
            .retain(|(write_row, _), _| *write_row != old_ref);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn record(
    tick: i64,
    event_id: i64,
    target: MutationTarget,
    row: usize,
    axis: Option<usize>,
    target_row: Option<usize>,
    before: f32,
    after: f32,
    cause: Cause,
) -> MutationRecord {
    MutationRecord {
        tick,
        event_id,
        target,
        row,
        axis,
        target_row,
        before,
        after,
        delta: after - before,
        cause,
    }
}

pub fn push_coalesced_mutation(trace: &mut StepTrace, mut next: MutationRecord, eps_log: f32) {
    if let Some(idx) = trace.mutations.iter().position(|existing| {
        existing.target == next.target
            && existing.row == next.row
            && existing.axis == next.axis
            && existing.target_row == next.target_row
    }) {
        let existing = trace.mutations[idx].clone();
        next.before = existing.before;
        next.delta = next.after - existing.before;
        next.cause = choose_cause(existing.cause, next.cause);
        if next.delta.abs() < eps_log {
            trace.mutations.remove(idx);
        } else {
            trace.mutations[idx] = next;
        }
    } else {
        trace.mutations.push(next);
    }
}

fn push_coalesced(trace: &mut StepTrace, next: MutationRecord, eps_log: f32) {
    push_coalesced_mutation(trace, next, eps_log);
}

fn choose_cause(old: Cause, new: Cause) -> Cause {
    use Cause::*;
    match (&old, &new) {
        (ContradictionOpen, _) | (ContradictionClose, _) => old,
        (_, ContradictionOpen) | (_, ContradictionClose) => new,
        (_, FreeRow) => new,
        (_, Merge) => new,
        (Allocate, BaselineDecay) | (Allocate, ActivationDecay) => old,
        (Allocate, _) => old,
        (_, Write) | (_, VarianceUpdate) => new,
        (_, _) => new,
    }
}
