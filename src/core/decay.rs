use crate::core::contradiction::has_open;
use crate::core::hebb::{add_link_delta, link_map, prune_row};
use crate::core::state::{record, AmState};
use crate::core::trace::{Cause, MutationTarget, StepTrace};
use std::collections::{BTreeSet, BTreeSet as ProtectedSet};

pub fn decay_and_forget(state: &mut AmState, trace: &mut StepTrace) {
    let rows = state.allocated_rows_sorted();
    for row in &rows {
        let decayed_a = state.a[*row] * state.theta.lam_a;
        state.a_set_logged(*row, decayed_a, trace, Cause::ActivationDecay);
        let next_b = state.theta.lam_b * state.b[*row] + state.theta.rho_b * state.a[*row];
        state.b_set_logged(*row, next_b, trace, Cause::BaselineDecay);
    }

    let candidates: Vec<usize> = state
        .allocated_rows_sorted()
        .into_iter()
        .filter(|row| should_gc(state, *row))
        .collect();
    let mut protected: ProtectedSet<usize> = ProtectedSet::new();
    for row in candidates {
        if protected.contains(&row) || !state.is_allocated(row) || !should_gc(state, row) {
            continue;
        }
        if let Some(target) = nearest_axis_neighbor(state, row) {
            merge_rows(state, row, target, trace);
            protected.insert(target);
        } else {
            free_row(state, row, trace, Cause::GarbageCollect);
        }
    }
}

fn should_gc(state: &AmState, row: usize) -> bool {
    state.tick - state.u[row] > state.theta.a_old
        && state.b[row] < state.theta.th_gc
        && !state.goals.contains(&row)
        && !(0..state.theta.d).any(|axis| has_open(state, row, axis))
}

fn nearest_axis_neighbor(state: &AmState, row: usize) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for other in state.allocated_rows_sorted() {
        if other == row {
            continue;
        }
        let sim = cosine(state, row, other);
        if sim > state.theta.th_merge
            && best
                .as_ref()
                .map(|(_, best_sim)| sim > *best_sim)
                .unwrap_or(true)
        {
            best = Some((other, sim));
        }
    }
    best.map(|(other, _)| other)
}

fn cosine(state: &AmState, left: usize, right: usize) -> f32 {
    let mut dot = 0.0;
    let mut ln = 0.0;
    let mut rn = 0.0;
    for axis in 0..state.theta.d {
        let l = state.m_get(left, axis);
        let r = state.m_get(right, axis);
        dot += l * r;
        ln += l * l;
        rn += r * r;
    }
    if ln <= state.theta.eps_log || rn <= state.theta.eps_log {
        return 0.0;
    }
    dot / (ln.sqrt() * rn.sqrt())
}

fn merge_rows(state: &mut AmState, row: usize, target: usize, trace: &mut StepTrace) {
    let before_links = link_map(state);
    let denom = state.b[target] + state.b[row];
    for axis in 0..state.theta.d {
        let after = if denom.abs() < state.theta.eps_log {
            (state.m_get(target, axis) + state.m_get(row, axis)) * 0.5
        } else {
            (state.b[target] * state.m_get(target, axis) + state.b[row] * state.m_get(row, axis))
                / denom
        };
        state.m_set_logged(target, axis, after, trace, Cause::Merge);
    }

    let outgoing = state.links[row].clone();
    for link in outgoing {
        if link.target != target && link.target != row {
            let current = state.links[target]
                .iter()
                .find(|existing| existing.target == link.target)
                .map(|existing| existing.weight)
                .unwrap_or(0.0);
            if link.weight > current {
                add_link_delta(state, target, link.target, link.weight - current);
            }
        }
    }

    for source in 0..state.theta.n {
        let mut redirected = 0.0_f32;
        state.links[source].retain(|link| {
            if link.target == row {
                redirected = redirected.max(link.weight);
                false
            } else {
                true
            }
        });
        if redirected > 0.0 && source != target {
            let current = state.links[source]
                .iter()
                .find(|existing| existing.target == target)
                .map(|existing| existing.weight)
                .unwrap_or(0.0);
            if redirected > current {
                add_link_delta(state, source, target, redirected - current);
            }
        }
        prune_row(state, source);
    }
    prune_row(state, target);

    let after_links = link_map(state);
    let keys: BTreeSet<(usize, usize)> = before_links
        .keys()
        .chain(after_links.keys())
        .copied()
        .collect();
    for (source, dest) in keys {
        let before = before_links.get(&(source, dest)).copied().unwrap_or(0.0);
        let after = after_links.get(&(source, dest)).copied().unwrap_or(0.0);
        if (after - before).abs() >= state.theta.eps_log {
            trace.mutations.push(record(
                state.tick,
                trace.event_id,
                MutationTarget::W,
                source,
                None,
                Some(dest),
                before,
                after,
                Cause::Merge,
            ));
        }
    }

    state.aliases.insert(row, target);
    state.log_nonscalar(
        trace,
        MutationTarget::Alias,
        row,
        Some(target),
        Cause::Merge,
    );
    free_row(state, row, trace, Cause::Merge);
}

fn free_row(state: &mut AmState, row: usize, trace: &mut StepTrace, cause: Cause) {
    for axis in 0..state.theta.d {
        state.m_set_logged(row, axis, 0.0, trace, Cause::FreeRow);
        state.v_set_logged(row, axis, 0.0, trace, Cause::FreeRow);
    }
    state.a_set_logged(row, 0.0, trace, Cause::FreeRow);
    state.b_set_logged(row, 0.0, trace, Cause::FreeRow);
    state.u_set_logged(row, 0, trace, Cause::FreeRow);
    state.links[row].clear();
    if let Some(label) = state.labels[row].take() {
        if cause == Cause::Merge {
            state.label_to_id.insert(label, row);
        } else {
            state.label_to_id.remove(&label);
        }
    }
    state.goals.retain(|goal| *goal != row);
    state
        .recent_writes
        .retain(|(write_row, _), _| *write_row != row);
    if !state.free_list.contains(&row) {
        state.free_list.push(row);
        state.free_list.sort_by(|left, right| right.cmp(left));
    }
    state.log_nonscalar(trace, MutationTarget::Free, row, None, cause);
}
