use crate::core::contradiction::update_contradictions;
use crate::core::decay::decay_and_forget;
use crate::core::event::Event;
use crate::core::hebb::update_links;
use crate::core::resolve::{apply_goal_ops, p1_resolve_and_cue};
use crate::core::settle::settle;
use crate::core::state::AmState;
use crate::core::trace::StepTrace;
use crate::core::write::write_asserts;
use anyhow::Result;
use std::collections::BTreeSet;

pub fn step(state: &mut AmState, event: &Event) -> StepTrace {
    step_result(state, event).expect("validated event should step")
}

pub fn step_result(state: &mut AmState, event: &Event) -> Result<StepTrace> {
    let mut event = event.clone();
    event.validate_and_clamp()?;
    validate_explicit_row_refs(state, &event)?;
    state.tick += 1;
    let mut trace = StepTrace::new(state.tick, event.id);
    let mut written_cells: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut external_input = vec![0.0; state.theta.n];

    p1_resolve_and_cue(
        state,
        &event,
        &mut trace,
        &mut written_cells,
        &mut external_input,
    )?;
    apply_goal_ops(state, &event, &mut trace)?;

    let settle_result = settle(state, &external_input, &mut trace);
    trace.settle_iters = settle_result.iterations;
    trace.active_set_before_decay = settle_result.active_set.clone();

    write_asserts(state, &event, &mut trace, &mut written_cells)?;
    update_links(state, &event, &settle_result.active_set, &mut trace)?;
    update_contradictions(state, &written_cells, &mut trace);
    decay_and_forget(state, &mut trace);
    Ok(trace)
}

fn validate_explicit_row_refs(state: &AmState, event: &Event) -> Result<()> {
    for cue in &event.cues {
        validate_ref(state, &cue.concept)?;
    }
    for assert in &event.asserts {
        validate_ref(state, &assert.concept)?;
    }
    for link in &event.links {
        validate_ref(state, &link.left)?;
        validate_ref(state, &link.right)?;
    }
    for op in &event.goal_ops {
        match op {
            crate::core::event::GoalOp::Push(reference)
            | crate::core::event::GoalOp::Pop(reference) => validate_ref(state, reference)?,
        }
    }
    Ok(())
}

fn validate_ref(state: &AmState, reference: &crate::core::event::ConceptRef) -> Result<()> {
    if matches!(reference, crate::core::event::ConceptRef::Id(_)) {
        state.resolve_existing(reference)?;
    }
    Ok(())
}
