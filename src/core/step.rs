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
    state.tick += 1;
    let mut trace = StepTrace::new(state.tick, event.id);
    let mut written_cells: BTreeSet<(usize, usize)> = BTreeSet::new();

    p1_resolve_and_cue(state, &event, &mut trace, &mut written_cells)?;
    apply_goal_ops(state, &event, &mut trace)?;

    let settle_result = settle(state, &mut trace);
    trace.settle_iters = settle_result.iterations;
    trace.active_set_before_decay = settle_result.active_set.clone();

    write_asserts(state, &event, &mut trace, &mut written_cells)?;
    update_links(state, &event, &settle_result.active_set, &mut trace)?;
    update_contradictions(state, &written_cells, &mut trace);
    decay_and_forget(state, &mut trace);
    Ok(trace)
}
