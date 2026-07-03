use crate::core::axes::axis_index;
use crate::core::event::{ConceptRef, Event, GoalOp};
use crate::core::state::AmState;
use crate::core::trace::{Cause, MutationTarget, StepTrace};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};

pub fn resolve_or_allocate(
    state: &mut AmState,
    reference: &ConceptRef,
    initial_targets: Option<&BTreeMap<String, f32>>,
    trace: &mut StepTrace,
    allocation_writes: &mut BTreeSet<(usize, usize)>,
) -> Result<usize> {
    match reference {
        ConceptRef::Id(row) => state.resolve_existing(&ConceptRef::Id(*row)),
        ConceptRef::Label(label) => {
            if let Some(row_ref) = state.label_to_id.get(label).copied() {
                let resolved = state.resolve_row_ref(row_ref)?;
                return Ok(resolved.id);
            }
            allocate_label(state, label, initial_targets, trace, allocation_writes)
        }
    }
}

pub fn p1_resolve_and_cue(
    state: &mut AmState,
    event: &Event,
    trace: &mut StepTrace,
    allocation_writes: &mut BTreeSet<(usize, usize)>,
    external_input: &mut [f32],
) -> Result<()> {
    for cue in &event.cues {
        let row = resolve_or_allocate(state, &cue.concept, None, trace, allocation_writes)?;
        external_input[row] = external_input[row].max(state.theta.k_i * cue.strength);
        if cue.strength > state.a[row] {
            state.a_set_logged(row, cue.strength, trace, Cause::Cue);
        }
        state.u_set_logged(row, state.tick, trace, Cause::Cue);
    }

    for assert in &event.asserts {
        let row = resolve_or_allocate(
            state,
            &assert.concept,
            Some(&assert.targets),
            trace,
            allocation_writes,
        )?;
        external_input[row] = external_input[row].max(state.theta.k_i * state.theta.a_init);
        if state.theta.a_init > state.a[row] {
            state.a_set_logged(row, state.theta.a_init, trace, Cause::AssertActivate);
        }
        state.u_set_logged(row, state.tick, trace, Cause::AssertActivate);
    }

    for link in &event.links {
        resolve_or_allocate(state, &link.left, None, trace, allocation_writes)?;
        resolve_or_allocate(state, &link.right, None, trace, allocation_writes)?;
    }

    for op in &event.goal_ops {
        match op {
            GoalOp::Push(reference) | GoalOp::Pop(reference) => {
                resolve_or_allocate(state, reference, None, trace, allocation_writes)?;
            }
        }
    }

    Ok(())
}

pub fn apply_goal_ops(state: &mut AmState, event: &Event, trace: &mut StepTrace) -> Result<()> {
    for op in &event.goal_ops {
        match op {
            GoalOp::Push(reference) => {
                let row = state.resolve_existing(reference)?;
                let row_ref = state.row_ref(row);
                if !state.goals.contains(&row_ref) {
                    state.goals.push(row_ref);
                    state.goals.sort_unstable();
                    state.log_nonscalar(trace, MutationTarget::Goal, row, None, Cause::GoalPush);
                }
            }
            GoalOp::Pop(reference) => {
                let row = state.resolve_existing(reference)?;
                let row_ref = state.row_ref(row);
                let before = state.goals.len();
                state.goals.retain(|goal| *goal != row_ref);
                if state.goals.len() != before {
                    state.log_nonscalar(trace, MutationTarget::Goal, row, None, Cause::GoalPop);
                }
            }
        }
    }
    Ok(())
}

fn allocate_label(
    state: &mut AmState,
    label: &str,
    initial_targets: Option<&BTreeMap<String, f32>>,
    trace: &mut StepTrace,
    allocation_writes: &mut BTreeSet<(usize, usize)>,
) -> Result<usize> {
    let row = state.free_list.pop().context("no free AM rows left")?;
    state.increment_generation_logged(row, trace, Cause::RowGeneration);
    state.allocated_set_logged(row, true, trace, Cause::Allocate);
    state.labels[row] = Some(label.to_string());
    state
        .label_to_id
        .insert(label.to_string(), state.row_ref(row));
    state.links[row].clear();
    state.log_nonscalar(trace, MutationTarget::Label, row, None, Cause::Allocate);

    for axis in 0..state.theta.d {
        state.v_set_logged(row, axis, state.theta.v0, trace, Cause::Allocate);
    }
    state.b_set_logged(row, state.theta.b0, trace, Cause::Allocate);
    state.u_set_logged(row, state.tick, trace, Cause::Allocate);

    if let Some(targets) = initial_targets {
        for (axis_name, value) in targets {
            let axis =
                axis_index(axis_name).with_context(|| format!("unknown axis {axis_name}"))?;
            let before = 0.0;
            state.m_set_logged(row, axis, *value, trace, Cause::Allocate);
            let delta = *value - before;
            if delta.abs() >= state.theta.eps_log {
                let sign = if delta > 0.0 { 1 } else { -1 };
                state.push_write_evidence(row, axis, trace.event_id, sign, delta);
                allocation_writes.insert((row, axis));
            }
        }
    }

    Ok(row)
}
