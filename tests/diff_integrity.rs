mod common;

use am001::core::event::Event;
use am001::core::hebb::link_map;
use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::core::trace::{MutationRecord, MutationTarget};
use common::{assert_event, cue_event, link_event, theta_default};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Cell {
    M(usize, usize),
    V(usize, usize),
    A(usize),
    B(usize),
    U(usize),
    W(usize, usize),
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
        let before = before_links.get(&(row, target)).copied().unwrap_or(0.0);
        let after = after_links.get(&(row, target)).copied().unwrap_or(0.0);
        if (before - after).abs() >= after_state_eps(after_links, before_links) {
            out.insert(Cell::W(row, target));
        }
    }
    out
}

fn after_state_eps(
    _after_links: &BTreeMap<(usize, usize), f32>,
    _before_links: &BTreeMap<(usize, usize), f32>,
) -> f32 {
    1e-6
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
            _ => continue,
        };
        *out.entry(cell).or_insert(0) += 1;
    }
    out
}
