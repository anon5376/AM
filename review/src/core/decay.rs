use crate::core::contradiction::has_open;
use crate::core::hebb::prune_row;
use crate::core::state::AmState;
use crate::core::trace::{Cause, MutationTarget, StepTrace};
use std::collections::{BTreeMap, BTreeSet, BTreeSet as ProtectedSet};

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
        && !state.goals.iter().any(|goal| {
            state
                .resolve_row_ref(*goal)
                .map(|goal_ref| goal_ref.id == row)
                .unwrap_or(false)
        })
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

    let mut touched_sources = BTreeSet::new();
    let outgoing = state.links[row].clone();
    for link in outgoing {
        if link.target != target && link.target != row {
            let current = link_weight(state, target, link.target);
            if link.weight > current {
                set_link_weight_logged(
                    state,
                    target,
                    link.target,
                    link.weight,
                    trace,
                    Cause::Merge,
                );
                touched_sources.insert(target);
            }
        }
    }

    for source in 0..state.theta.n {
        let redirected = link_weight(state, source, row);
        if redirected > 0.0 {
            set_link_weight_logged(state, source, row, 0.0, trace, Cause::Merge);
            touched_sources.insert(source);
        }
        if redirected > 0.0 && source != target && source != row {
            let current = link_weight(state, source, target);
            if redirected > current {
                set_link_weight_logged(state, source, target, redirected, trace, Cause::Merge);
                touched_sources.insert(source);
            }
        }
    }

    for source in touched_sources {
        if state.is_allocated(source) {
            prune_row_logged(state, source, trace, Cause::Merge);
        }
    }

    let old_ref = state.row_ref(row);
    let target_ref = state.row_ref(target);
    state.aliases.insert(old_ref, target_ref);
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
    let old_ref = state.row_ref(row);
    remove_outgoing_links_logged(state, row, trace, cause);
    remove_inbound_links_logged(state, row, trace, cause);
    for axis in 0..state.theta.d {
        state.m_set_logged(row, axis, 0.0, trace, Cause::FreeRow);
        state.v_set_logged(row, axis, 0.0, trace, Cause::FreeRow);
    }
    state.a_set_logged(row, 0.0, trace, Cause::FreeRow);
    state.b_set_logged(row, 0.0, trace, Cause::FreeRow);
    state.u_set_logged(row, 0, trace, Cause::FreeRow);
    state.allocated_set_logged(row, false, trace, Cause::FreeRow);
    state.clear_free_metadata(row, old_ref, cause);
    if !state.free_list.contains(&row) {
        state.free_list.push(row);
        state.free_list.sort_by(|left, right| right.cmp(left));
    }
    state.increment_generation_logged(row, trace, Cause::RowGeneration);
    state.log_nonscalar(trace, MutationTarget::Free, row, None, cause);
}

fn link_weight(state: &AmState, source: usize, target: usize) -> f32 {
    state.links[source]
        .iter()
        .find(|link| link.target == target)
        .map(|link| link.weight)
        .unwrap_or(0.0)
}

fn set_link_weight_logged(
    state: &mut AmState,
    source: usize,
    target: usize,
    after: f32,
    trace: &mut StepTrace,
    cause: Cause,
) {
    if source == target {
        return;
    }
    let before = link_weight(state, source, target);
    let after = after.clamp(0.0, state.theta.w_max);
    let after = if after < state.theta.eps_w {
        0.0
    } else {
        after
    };
    if (after - before).abs() < state.theta.eps_log {
        return;
    }
    if after == 0.0 {
        state.links[source].retain(|link| link.target != target);
    } else if let Some(link) = state.links[source]
        .iter_mut()
        .find(|link| link.target == target)
    {
        link.weight = after;
    } else {
        state.links[source].push(crate::core::state::Link {
            target,
            weight: after,
        });
    }
    state.links[source].sort_by_key(|link| link.target);
    state.log_w_change(trace, source, target, before, after, cause);
}

fn remove_outgoing_links_logged(
    state: &mut AmState,
    row: usize,
    trace: &mut StepTrace,
    cause: Cause,
) {
    let targets: Vec<usize> = state.links[row].iter().map(|link| link.target).collect();
    for target in targets {
        set_link_weight_logged(state, row, target, 0.0, trace, cause);
    }
}

fn remove_inbound_links_logged(
    state: &mut AmState,
    row: usize,
    trace: &mut StepTrace,
    cause: Cause,
) {
    for source in 0..state.theta.n {
        if source == row {
            continue;
        }
        if link_weight(state, source, row) > 0.0 {
            set_link_weight_logged(state, source, row, 0.0, trace, cause);
        }
    }
}

fn prune_row_logged(state: &mut AmState, row: usize, trace: &mut StepTrace, cause: Cause) {
    let before = row_link_map(state, row);
    prune_row(state, row);
    let after = row_link_map(state, row);
    let targets = before
        .keys()
        .chain(after.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for target in targets {
        let before_weight = before.get(&target).copied().unwrap_or(0.0);
        let after_weight = after.get(&target).copied().unwrap_or(0.0);
        state.log_w_change(trace, row, target, before_weight, after_weight, cause);
    }
}

fn row_link_map(state: &AmState, row: usize) -> BTreeMap<usize, f32> {
    state.links[row]
        .iter()
        .map(|link| (link.target, link.weight))
        .collect()
}
