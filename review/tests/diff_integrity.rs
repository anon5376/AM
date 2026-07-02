mod common;

use am001::core::event::Event;
use am001::core::hebb::{add_link_delta, link_map, update_links};
use am001::core::snapshot::from_bytes;
use am001::core::state::{AmState, Link};
use am001::core::step::step_result;
use am001::core::trace::{MutationRecord, MutationTarget, StepTrace};
use common::{assert_event, cue_event, link_event, row, theta_default};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Cell {
    M(usize, usize),
    V(usize, usize),
    A(usize),
    B(usize),
    U(usize),
    W(usize, usize),
    Generation(usize),
    Allocated(usize),
}

#[test]
fn every_changed_scalar_and_link_has_exactly_one_trace_record() {
    let mut state = AmState::new(theta_default());
    let mut events = Vec::new();
    for idx in 0..20 {
        events.push(assert_event(
            idx + 1,
            &format!("c{idx}"),
            &[
                ("truth_assert", if idx % 2 == 0 { 1.0 } else { -1.0 }),
                ("goal_relevance", (idx as f32 % 5.0) / 5.0),
            ],
        ));
    }
    for idx in 0..10 {
        events.push(link_event(
            100 + idx,
            &format!("c{idx}"),
            &format!("c{}", idx + 1),
            1.0,
        ));
        events.push(cue_event(200 + idx, &format!("c{idx}"), 0.7));
    }
    for idx in 0..20 {
        events.push(assert_event(
            300 + idx,
            &format!("c{}", idx % 5),
            &[("truth_assert", if idx % 2 == 0 { -1.0 } else { 1.0 })],
        ));
    }

    for event in events.into_iter().take(50) {
        check_one_step(&mut state, &event);
    }
}

#[test]
fn diff_integrity_covers_contradiction_free_reuse_merge_and_sub_eps_w_decay() {
    contradiction_open_and_close_records_match_final_changes();
    free_reuse_and_snapshot_roundtrip_records_match_final_changes();
    merge_inbound_and_outgoing_records_match_final_changes();
    sub_eps_w_decay_is_not_applied_or_logged();
}

#[test]
fn sub_eps_w_growth_and_saturation_are_not_applied_or_logged() {
    let mut theta = theta_default();
    theta.eps_log = 0.01;
    theta.eps_w = 0.000001;
    theta.eta_w = 1.0;
    theta.del_w = 0.0;
    let mut state = AmState::new(theta);

    add_link_delta(&mut state, 0, 1, 0.005);
    assert!(state.links[0].is_empty());

    check_one_step(&mut state, &assert_event(1, "a", &[("truth_assert", 1.0)]));
    check_one_step(&mut state, &assert_event(2, "b", &[("agency", 1.0)]));
    let a = row(&state, "a");
    let b = row(&state, "b");
    state.links[a].clear();
    state.links[b].clear();
    state.links[a].push(Link {
        target: b,
        weight: 0.995,
    });
    state.links[b].push(Link {
        target: a,
        weight: 0.995,
    });

    let mut trace = StepTrace::new(state.tick + 1, 3);
    update_links(&mut state, &link_event(3, "a", "b", 1.0), &[], &mut trace).unwrap();
    assert_eq!(state.links[a][0].weight, 0.995);
    assert_eq!(state.links[b][0].weight, 0.995);
    assert!(trace
        .mutations
        .iter()
        .all(|mutation| mutation.target != MutationTarget::W));
}

fn check_one_step(state: &mut AmState, event: &Event) {
    let before = state.clone();
    let before_links = link_map(&before);
    let trace = step_result(state, event).unwrap();
    let after_links = link_map(state);
    let changed = changed_cells(&before, state, &before_links, &after_links);
    let records = record_cells(&trace.mutations);

    for cell in &changed {
        let count = records.get(cell).copied().unwrap_or(0);
        assert_eq!(count, 1, "changed cell {cell:?} had {count} records");
    }
    for (cell, count) in records {
        assert_eq!(count, 1, "record cell {cell:?} had duplicate records");
        assert!(
            changed.contains(&cell),
            "record without final change: {cell:?}"
        );
    }
}

fn changed_cells(
    before: &AmState,
    after: &AmState,
    before_links: &BTreeMap<(usize, usize), f32>,
    after_links: &BTreeMap<(usize, usize), f32>,
) -> BTreeSet<Cell> {
    let mut out = BTreeSet::new();
    for row in 0..after.theta.n {
        if before.generation[row] != after.generation[row] {
            out.insert(Cell::Generation(row));
        }
        if before.allocated[row] != after.allocated[row] {
            out.insert(Cell::Allocated(row));
        }
        if (before.a[row] - after.a[row]).abs() >= after.theta.eps_log {
            out.insert(Cell::A(row));
        }
        if (before.b[row] - after.b[row]).abs() >= after.theta.eps_log {
            out.insert(Cell::B(row));
        }
        if before.u[row] != after.u[row] {
            out.insert(Cell::U(row));
        }
        for axis in 0..after.theta.d {
            if (before.m[before.idx(row, axis)] - after.m[after.idx(row, axis)]).abs()
                >= after.theta.eps_log
            {
                out.insert(Cell::M(row, axis));
            }
            if (before.v[before.idx(row, axis)] - after.v[after.idx(row, axis)]).abs()
                >= after.theta.eps_log
            {
                out.insert(Cell::V(row, axis));
            }
        }
    }
    let keys = before_links
        .keys()
        .chain(after_links.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for (row, target) in keys {
        let before_weight = before_links.get(&(row, target)).copied().unwrap_or(0.0);
        let after_weight = after_links.get(&(row, target)).copied().unwrap_or(0.0);
        if (before_weight - after_weight).abs() >= after.theta.eps_log {
            out.insert(Cell::W(row, target));
        }
    }
    out
}

fn record_cells(records: &[MutationRecord]) -> BTreeMap<Cell, usize> {
    let mut out = BTreeMap::new();
    for record in records {
        let cell = match record.target {
            MutationTarget::M => Cell::M(record.row, record.axis.unwrap()),
            MutationTarget::V => Cell::V(record.row, record.axis.unwrap()),
            MutationTarget::A => Cell::A(record.row),
            MutationTarget::B => Cell::B(record.row),
            MutationTarget::U => Cell::U(record.row),
            MutationTarget::W => Cell::W(record.row, record.target_row.unwrap()),
            MutationTarget::Generation => Cell::Generation(record.row),
            MutationTarget::Allocation => Cell::Allocated(record.row),
            _ => continue,
        };
        *out.entry(cell).or_insert(0) += 1;
    }
    out
}

fn contradiction_open_and_close_records_match_final_changes() {
    let mut state = AmState::new(theta_default());
    for id in 1..=4 {
        let value = if id % 2 == 1 { 1.0 } else { -1.0 };
        check_one_step(
            &mut state,
            &assert_event(id, "rust", &[("truth_assert", value)]),
        );
    }
    assert!(!state.open_contradictions.is_empty());
    for id in 5..=24 {
        check_one_step(
            &mut state,
            &assert_event(id, "rust", &[("truth_assert", 1.0)]),
        );
    }
    assert!(state
        .open_contradictions
        .iter()
        .any(|contradiction| matches!(
            contradiction.status,
            am001::core::state::ContradictionStatus::Closed { .. }
        )));
}

fn free_reuse_and_snapshot_roundtrip_records_match_final_changes() {
    let mut theta = theta_default();
    theta.th_merge = 2.0;
    theta.del_w = 0.0;
    let mut state = AmState::new(theta);

    check_one_step(&mut state, &assert_event(1, "a", &[("truth_assert", 1.0)]));
    check_one_step(&mut state, &assert_event(2, "b", &[("agency", 1.0)]));
    let a = row(&state, "a");
    let b = row(&state, "b");
    state.links[a].push(Link {
        target: b,
        weight: 0.45,
    });
    state.links[b].push(Link {
        target: a,
        weight: 0.35,
    });
    state.b[b] = 0.0;
    state.a[b] = 0.0;
    state.force_last_touched(b, state.tick - state.theta.a_old - 1);
    check_one_step(&mut state, &Event::empty(3));
    assert!(!state.is_allocated(b));
    assert!(state.links[a].iter().all(|link| link.target != b));

    for value in &mut state.a {
        *value = 0.0;
    }
    state.theta.eta_w = 0.0;
    check_one_step(
        &mut state,
        &assert_event(4, "c", &[("goal_relevance", 1.0)]),
    );
    let c = row(&state, "c");
    assert_eq!(c, b);
    assert!(state.links[a].iter().all(|link| link.target != c));

    let bytes = state.snapshot_bytes();
    let loaded = from_bytes(&bytes).unwrap();
    assert_eq!(bytes, loaded.snapshot_bytes());
}

fn merge_inbound_and_outgoing_records_match_final_changes() {
    let mut theta = theta_default();
    theta.del_w = 0.0;
    theta.th_merge = 0.90;
    let mut state = AmState::new(theta);

    check_one_step(&mut state, &assert_event(1, "m1", &[("truth_assert", 0.9)]));
    check_one_step(&mut state, &assert_event(2, "m2", &[("truth_assert", 0.9)]));
    check_one_step(&mut state, &assert_event(3, "sink", &[("agency", 0.9)]));
    let m1 = row(&state, "m1");
    let m2 = row(&state, "m2");
    let sink = row(&state, "sink");
    let m1_ref = state.row_ref(m1);
    state.links[m1].push(Link {
        target: sink,
        weight: 0.50,
    });
    state.links[sink].push(Link {
        target: m1,
        weight: 0.40,
    });
    state.b[m1] = 0.0;
    state.a[m1] = 0.0;
    state.force_last_touched(m1, state.tick - state.theta.a_old - 1);

    check_one_step(&mut state, &Event::empty(4));
    assert!(!state.is_allocated(m1));
    assert_eq!(state.aliases.get(&m1_ref), Some(&state.row_ref(m2)));
    assert!(state.links[m1].is_empty());
    assert!(state.links[sink].iter().all(|link| link.target != m1));
    assert!(state.links[m2].iter().any(|link| link.target == sink));
    assert!(state.links[sink].iter().any(|link| link.target == m2));

    let bytes = state.snapshot_bytes();
    let loaded = from_bytes(&bytes).unwrap();
    assert_eq!(bytes, loaded.snapshot_bytes());
}

fn sub_eps_w_decay_is_not_applied_or_logged() {
    let mut theta = theta_default();
    theta.eps_log = 0.01;
    theta.eps_w = 0.000001;
    theta.del_w = 0.001;
    theta.a_old = 1_000_000;
    let mut state = AmState::new(theta);

    check_one_step(&mut state, &assert_event(1, "a", &[("truth_assert", 1.0)]));
    check_one_step(&mut state, &assert_event(2, "b", &[("agency", 1.0)]));
    let a = row(&state, "a");
    let b = row(&state, "b");
    state.links[a].clear();
    state.links[b].clear();
    state.links[a].push(Link {
        target: b,
        weight: 0.5,
    });
    state.theta.eta_w = 0.0;
    let before = link_map(&state).get(&(a, b)).copied().unwrap();

    for id in 3..=80 {
        let trace = step_result(&mut state, &Event::empty(id)).unwrap();
        assert!(trace
            .mutations
            .iter()
            .all(|mutation| mutation.target != MutationTarget::W));
    }
    let after = link_map(&state).get(&(a, b)).copied().unwrap();
    assert_eq!(before, after);
}
