use crate::core::state::{AmState, ContradictionStatus, OpenContradiction};
use crate::core::trace::{Cause, MutationTarget, StepTrace};
use std::collections::BTreeSet;

pub fn update_contradictions(
    state: &mut AmState,
    written_cells: &BTreeSet<(usize, usize)>,
    trace: &mut StepTrace,
) {
    for (row, axis) in written_cells {
        if state.v_get(*row, *axis) > state.theta.th_v
            && signs_alternate(state, *row, *axis)
            && !has_open(state, *row, *axis)
        {
            let evidence_event_ids = state
                .recent_writes
                .get(&(*row, *axis))
                .map(|history| history.iter().map(|item| item.event_id).collect())
                .unwrap_or_default();
            let opened = OpenContradiction {
                concept: *row,
                axis: *axis,
                evidence_event_ids,
                opened_tick: state.tick,
                pressure: state.theta.rho_c,
                status: ContradictionStatus::Open,
            };
            state.open_contradictions.push(opened.clone());
            let new_b = state.b[*row] + state.theta.rho_c;
            state.b_set_logged(*row, new_b, trace, Cause::ContradictionOpen);
            state.log_nonscalar(
                trace,
                MutationTarget::Contradiction,
                *row,
                Some(*axis),
                Cause::ContradictionOpen,
            );
            trace.opened_contradictions.push(opened);
        }
    }

    let mut close_indices = Vec::new();
    for (idx, contradiction) in state.open_contradictions.iter().enumerate() {
        if contradiction.status == ContradictionStatus::Open
            && state.v_get(contradiction.concept, contradiction.axis) < state.theta.th_v_close
        {
            close_indices.push(idx);
        }
    }

    for idx in close_indices {
        let row = state.open_contradictions[idx].concept;
        let axis = state.open_contradictions[idx].axis;
        state.open_contradictions[idx].status = ContradictionStatus::Closed {
            closed_tick: state.tick,
        };
        let closed = state.open_contradictions[idx].clone();
        let new_b = state.b[row] - state.theta.rho_c;
        state.b_set_logged(row, new_b, trace, Cause::ContradictionClose);
        state.log_nonscalar(
            trace,
            MutationTarget::Contradiction,
            row,
            Some(axis),
            Cause::ContradictionClose,
        );
        trace.closed_contradictions.push(closed);
    }
}

pub fn has_open(state: &AmState, row: usize, axis: usize) -> bool {
    state.open_contradictions.iter().any(|contradiction| {
        contradiction.concept == row
            && contradiction.axis == axis
            && contradiction.status == ContradictionStatus::Open
    })
}

fn signs_alternate(state: &AmState, row: usize, axis: usize) -> bool {
    let Some(history) = state.recent_writes.get(&(row, axis)) else {
        return false;
    };
    if history.len() < 4 {
        return false;
    }
    let take = history.len().min(state.theta.m_sign_window);
    let start = history.len() - take;
    history[start..]
        .windows(2)
        .all(|pair| pair[0].sign != 0 && pair[1].sign != 0 && pair[0].sign == -pair[1].sign)
}
