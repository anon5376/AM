use crate::core::event::Event;
use crate::core::state::{AmState, Link};
use crate::core::trace::{Cause, StepTrace};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

pub fn update_links(
    state: &mut AmState,
    event: &Event,
    active_set: &[usize],
    trace: &mut StepTrace,
) -> Result<()> {
    let before = link_map(state);
    let mut caused: BTreeMap<(usize, usize), Cause> = BTreeMap::new();

    if !active_set.is_empty() {
        for i in active_set {
            for j in active_set {
                if i == j {
                    continue;
                }
                let delta = state.theta.eta_w * state.a[*i] * state.a[*j];
                if delta.abs() >= state.theta.eps_log {
                    add_link_delta(state, *i, *j, delta);
                    caused.entry((*i, *j)).or_insert(Cause::Hebb);
                }
            }
        }
    }

    for link in &event.links {
        let left = state.resolve_existing(&link.left)?;
        let right = state.resolve_existing(&link.right)?;
        if left == right {
            continue;
        }
        let delta = state.theta.eta_w * link.hint;
        if delta.abs() >= state.theta.eps_log {
            add_link_delta(state, left, right, delta);
            add_link_delta(state, right, left, delta);
            caused.insert((left, right), Cause::LinkHint);
            caused.insert((right, left), Cause::LinkHint);
        }
    }

    for row in 0..state.theta.n {
        if state.links[row].is_empty() {
            continue;
        }
        for link in &mut state.links[row] {
            let decayed = link.weight * (1.0 - state.theta.del_w);
            if (decayed - link.weight).abs() >= state.theta.eps_log {
                link.weight = decayed;
            }
        }
        prune_row(state, row);
    }

    let after = link_map(state);
    let keys: BTreeSet<(usize, usize)> = before.keys().chain(after.keys()).copied().collect();
    for (row, target) in keys {
        let old = before.get(&(row, target)).copied().unwrap_or(0.0);
        let new = after.get(&(row, target)).copied().unwrap_or(0.0);
        if (new - old).abs() < state.theta.eps_log {
            continue;
        }
        let cause = caused.get(&(row, target)).cloned().unwrap_or({
            if new == 0.0 {
                Cause::LinkPrune
            } else {
                Cause::LinkDecay
            }
        });
        state.log_w_change(trace, row, target, old, new, cause);
    }

    Ok(())
}

pub fn link_map(state: &AmState) -> BTreeMap<(usize, usize), f32> {
    let mut out = BTreeMap::new();
    for (row, links) in state.links.iter().enumerate() {
        for link in links {
            out.insert((row, link.target), link.weight);
        }
    }
    out
}

pub fn add_link_delta(state: &mut AmState, row: usize, target: usize, delta: f32) {
    if row == target {
        return;
    }
    if let Some(link) = state.links[row]
        .iter_mut()
        .find(|link| link.target == target)
    {
        link.weight = (link.weight + delta).clamp(0.0, state.theta.w_max);
    } else {
        let weight = delta.clamp(0.0, state.theta.w_max);
        if weight >= state.theta.eps_w {
            state.links[row].push(Link { target, weight });
        }
    }
    state.links[row].sort_by_key(|link| link.target);
}

pub fn prune_row(state: &mut AmState, row: usize) {
    state.links[row].retain(|link| link.weight >= state.theta.eps_w && link.target != row);
    state.links[row].sort_by(|left, right| {
        right
            .weight
            .total_cmp(&left.weight)
            .then_with(|| left.target.cmp(&right.target))
    });
    if state.links[row].len() > state.theta.k {
        state.links[row].truncate(state.theta.k);
    }
    state.links[row].sort_by_key(|link| link.target);
}
