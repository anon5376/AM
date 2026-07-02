use crate::core::axes::axis_index;
use crate::core::diff::trace_hash;
use crate::core::event::{Assert, ConceptRef, Cue, Event, GoalOp, LinkAssert};
use crate::core::hebb::link_map;
use crate::core::snapshot::from_bytes;
use crate::core::state::{AmState, ContradictionStatus, OpenContradiction};
use crate::core::step::step_result;
use crate::core::theta::Theta;
use crate::core::trace::{MutationRecord, MutationTarget};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct CriterionResult {
    pub name: &'static str,
    pub passed: bool,
    pub metrics: BTreeMap<String, f64>,
    pub detail: String,
}

impl CriterionResult {
    fn pass(name: &'static str) -> Self {
        Self {
            name,
            passed: true,
            metrics: BTreeMap::new(),
            detail: String::new(),
        }
    }

    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            passed: false,
            metrics: BTreeMap::new(),
            detail: detail.into(),
        }
    }

    fn metric(mut self, metric_name: &str, value: f64) -> Self {
        self.metrics.insert(metric_name.to_string(), value);
        self
    }
}

pub fn run_core_criteria(theta: &Theta) -> Vec<CriterionResult> {
    vec![
        determinism(theta),
        completion(theta),
        reinforcement(theta),
        contradiction(theta),
        forgetting(theta),
        diff_integrity(theta),
    ]
}

pub fn determinism(theta: &Theta) -> CriterionResult {
    let events = vec![
        assert_event(1, "am001", &[("goal_relevance", 1.0), ("agency", 0.8)]),
        assert_event(2, "rust", &[("truth_assert", 1.0), ("goal_relevance", 0.8)]),
        link_event(3, "rust", "am001", 1.0),
        assert_event(4, "rust", &[("truth_assert", -1.0)]),
        assert_event(5, "rust", &[("truth_assert", 1.0)]),
    ];

    let mut left = AmState::new(theta.clone());
    let mut left_traces = Vec::new();
    for event in &events {
        left_traces.push(match step_result(&mut left, event) {
            Ok(trace) => trace,
            Err(err) => return CriterionResult::fail("determinism", err.to_string()),
        });
    }

    let mut right = AmState::new(theta.clone());
    let mut right_traces = Vec::new();
    for event in &events {
        right_traces.push(step_result(&mut right, event).expect("determinism replay"));
    }

    let state_hash = left.state_hash();
    let left_trace_hash = trace_hash(&left_traces).expect("trace hash");
    let same_direct = state_hash == right.state_hash()
        && left_trace_hash == trace_hash(&right_traces).expect("trace hash");

    let mut interrupted = AmState::new(theta.clone());
    let mut interrupted_traces = Vec::new();
    for event in &events[..3] {
        interrupted_traces.push(step_result(&mut interrupted, event).expect("midway step"));
    }
    let bytes = interrupted.snapshot_bytes();
    let mut continued = match from_bytes(&bytes) {
        Ok(state) => state,
        Err(err) => return CriterionResult::fail("determinism", err.to_string()),
    };
    for event in &events[3..] {
        interrupted_traces.push(step_result(&mut continued, event).expect("continued step"));
    }
    let same_snapshot = state_hash == continued.state_hash()
        && left_trace_hash == trace_hash(&interrupted_traces).expect("trace hash");

    if same_direct && same_snapshot {
        CriterionResult::pass("determinism")
    } else {
        CriterionResult::fail("determinism", "state or trace hash diverged")
    }
}

pub fn completion(theta: &Theta) -> CriterionResult {
    let mut state = AmState::new(theta.clone());
    let mut event_id = 1;
    let mut assemblies = Vec::new();

    for assembly in 0..30 {
        let labels = (0..5)
            .map(|member| format!("asm{assembly}_{member}"))
            .collect::<Vec<_>>();
        for label in &labels {
            let signature = if assembly % 2 == 0 { 1.0 } else { -1.0 };
            let axis_value = ((assembly as f32 % 7.0) - 3.0) / 3.0;
            let event = assert_event(
                event_id,
                label,
                &[
                    ("memory_relevance", signature),
                    ("project_relevance", axis_value),
                    ("truth_assert", 0.5),
                ],
            );
            if let Err(err) = step_result(&mut state, &event) {
                return CriterionResult::fail("completion", err.to_string());
            }
            event_id += 1;
        }
        for left in 0..5 {
            for right in (left + 1)..5 {
                for _ in 0..2 {
                    let event = link_event(event_id, &labels[left], &labels[right], 1.0);
                    if let Err(err) = step_result(&mut state, &event) {
                        return CriterionResult::fail("completion", err.to_string());
                    }
                    event_id += 1;
                }
            }
        }
        state.a.fill(0.0);
        assemblies.push(labels);
    }

    let trained = state.clone();
    let mut recall_sum = 0.0;
    let mut foreign_sum = 0.0;

    for labels in &assemblies {
        let mut trial = trained.clone();
        let _ = step_result(&mut trial, &Event::empty(event_id));
        event_id += 1;
        let trace = match step_result(
            &mut trial,
            &cue_two_event(event_id, &labels[0], &labels[1], 0.8),
        ) {
            Ok(trace) => trace,
            Err(err) => return CriterionResult::fail("completion", err.to_string()),
        };
        event_id += 1;
        let active = trace
            .active_set_before_decay
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let assembly_rows = labels
            .iter()
            .map(|label| row(&trial, label))
            .collect::<BTreeSet<_>>();
        let non_cued = labels[2..]
            .iter()
            .filter(|label| active.contains(&row(&trial, label)))
            .count();
        recall_sum += non_cued as f64 / 3.0;
        let foreign = active
            .iter()
            .filter(|row_id| !assembly_rows.contains(row_id))
            .count();
        foreign_sum += if active.is_empty() {
            0.0
        } else {
            foreign as f64 / active.len() as f64
        };
    }

    let recall = recall_sum / assemblies.len() as f64;
    let contamination = foreign_sum / assemblies.len() as f64;
    let passed = recall >= 0.90 && contamination < 0.05;
    let result = if passed {
        CriterionResult::pass("completion")
    } else {
        CriterionResult::fail(
            "completion",
            format!("recall={recall:.4}, contamination={contamination:.4}"),
        )
    };
    result
        .metric("recall", recall)
        .metric("contamination", contamination)
}

pub fn reinforcement(theta: &Theta) -> CriterionResult {
    let mut state = AmState::new(theta.clone());
    let truth = axis("truth_assert");
    let mut last_m = f32::NEG_INFINITY;
    let mut last_b = f32::NEG_INFINITY;

    for id in 1..=10 {
        if let Err(err) = step_result(
            &mut state,
            &assert_event(
                id,
                "rust",
                &[("truth_assert", 1.0), ("goal_relevance", 0.8)],
            ),
        ) {
            return CriterionResult::fail("reinforcement", err.to_string());
        }
        let rust = row(&state, "rust");
        let current_m = state.m_get(rust, truth);
        let current_b = state.b[rust];
        if current_m + state.theta.eps_log < last_m {
            return CriterionResult::fail("reinforcement", "M was not monotone");
        }
        if current_b + state.theta.eps_log < last_b {
            return CriterionResult::fail("reinforcement", "b was not monotone");
        }
        last_m = current_m;
        last_b = current_b;
    }

    let rust_rows = state
        .labels
        .iter()
        .filter(|label| label.as_deref() == Some("rust"))
        .count();
    let final_m = state.m_get(row(&state, "rust"), truth);
    if rust_rows == 1 && (final_m - 1.0).abs() < 0.001 {
        CriterionResult::pass("reinforcement").metric("final_m", final_m as f64)
    } else {
        CriterionResult::fail(
            "reinforcement",
            format!("rust_rows={rust_rows}, final_m={final_m}"),
        )
    }
}

pub fn contradiction(theta: &Theta) -> CriterionResult {
    let mut state = AmState::new(theta.clone());
    let truth = axis("truth_assert");

    for (id, value) in [(1, 1.0), (2, -1.0), (3, 1.0)] {
        if let Err(err) = step_result(
            &mut state,
            &assert_event(id, "rust", &[("truth_assert", value)]),
        ) {
            return CriterionResult::fail("contradiction", err.to_string());
        }
    }
    let trace = match step_result(
        &mut state,
        &assert_event(4, "rust", &[("truth_assert", -1.0)]),
    ) {
        Ok(trace) => trace,
        Err(err) => return CriterionResult::fail("contradiction", err.to_string()),
    };
    let rust = row(&state, "rust");
    let opened = !trace.opened_contradictions.is_empty()
        && state.v_get(rust, truth) > state.theta.th_v
        && state.m_get(rust, truth).abs() < 1.0
        && state.open_contradictions.iter().any(|contradiction| {
            contradiction.concept == state.row_ref(rust)
                && contradiction.axis == truth
                && contradiction.status == ContradictionStatus::Open
        });
    if !opened {
        return CriterionResult::fail("contradiction", "did not open on fourth alternating write")
            .metric("v", state.v_get(rust, truth) as f64);
    }

    for id in 5..=24 {
        if let Err(err) = step_result(
            &mut state,
            &assert_event(id, "rust", &[("truth_assert", 1.0)]),
        ) {
            return CriterionResult::fail("contradiction", err.to_string());
        }
    }
    let closed = state.open_contradictions.iter().any(|contradiction| {
        contradiction.concept == state.row_ref(rust)
            && contradiction.axis == truth
            && matches!(contradiction.status, ContradictionStatus::Closed { .. })
    });
    if closed {
        CriterionResult::pass("contradiction")
            .metric("v_after_open", state.v_get(rust, truth) as f64)
    } else {
        CriterionResult::fail("contradiction", "did not close after same-sign run")
    }
}

pub fn forgetting(theta: &Theta) -> CriterionResult {
    let mut state = AmState::new(theta.clone());
    let _ = step_result(
        &mut state,
        &assert_event(1, "old", &[("truth_assert", 0.4)]),
    );
    let old = row(&state, "old");
    state.b[old] = 0.0;
    state.a[old] = 0.0;
    state.force_last_touched(old, state.tick - state.theta.a_old - 1);
    let _ = step_result(&mut state, &Event::empty(2));
    if state.is_allocated(old) {
        return CriterionResult::fail("forgetting", "stale low-b row was not freed");
    }

    let mut merge_state = AmState::new(theta.clone());
    let _ = step_result(
        &mut merge_state,
        &assert_event(1, "m1", &[("truth_assert", 1.0)]),
    );
    let _ = step_result(
        &mut merge_state,
        &assert_event(2, "m2", &[("truth_assert", 0.99)]),
    );
    let m1 = row(&merge_state, "m1");
    let m2 = row(&merge_state, "m2");
    let m1_ref = merge_state.row_ref(m1);
    let m2_ref = merge_state.row_ref(m2);
    merge_state.b[m1] = 0.0;
    merge_state.b[m2] = 0.0;
    merge_state.a[m1] = 0.0;
    merge_state.a[m2] = 0.0;
    merge_state.force_last_touched(m1, merge_state.tick - merge_state.theta.a_old - 1);
    merge_state.force_last_touched(m2, merge_state.tick - merge_state.theta.a_old - 1);
    let _ = step_result(&mut merge_state, &Event::empty(3));
    if merge_state.aliases.get(&m1_ref) != Some(&m2_ref) || !merge_state.is_allocated(m2) {
        return CriterionResult::fail("forgetting", "similar stale rows did not merge");
    }

    let mut protected = AmState::new(theta.clone());
    let _ = step_result(
        &mut protected,
        &assert_event(1, "goalish", &[("goal_relevance", 1.0)]),
    );
    let goalish = row(&protected, "goalish");
    let goal_event = Event {
        id: 2,
        cues: Vec::new(),
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: vec![GoalOp::Push(ConceptRef::Label("goalish".to_string()))],
    };
    let _ = step_result(&mut protected, &goal_event);
    protected.b[goalish] = 0.0;
    protected.a[goalish] = 0.0;
    protected.force_last_touched(goalish, protected.tick - protected.theta.a_old - 1);
    let _ = step_result(&mut protected, &Event::empty(3));
    if !protected.is_allocated(goalish) {
        return CriterionResult::fail("forgetting", "goal row was garbage-collected");
    }

    protected.open_contradictions.push(OpenContradiction {
        concept: protected.row_ref(goalish),
        axis: axis("truth_assert"),
        evidence_event_ids: vec![1, 2, 3, 4],
        opened_tick: protected.tick,
        pressure: protected.theta.rho_c,
        status: ContradictionStatus::Open,
    });
    protected.goals.clear();
    protected.force_last_touched(goalish, protected.tick - protected.theta.a_old - 1);
    let _ = step_result(&mut protected, &Event::empty(4));
    if protected.is_allocated(goalish) {
        CriterionResult::pass("forgetting")
    } else {
        CriterionResult::fail("forgetting", "open contradiction row was garbage-collected")
    }
}

pub fn diff_integrity(theta: &Theta) -> CriterionResult {
    let mut state = AmState::new(theta.clone());
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
        if let Err(detail) = check_diff_one_step(&mut state, &event) {
            return CriterionResult::fail("diff_integrity", detail);
        }
    }
    CriterionResult::pass("diff_integrity")
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Cell {
    M(usize, usize),
    V(usize, usize),
    A(usize),
    B(usize),
    U(usize),
    W(usize, usize),
    Generation(usize),
}

fn check_diff_one_step(state: &mut AmState, event: &Event) -> Result<(), String> {
    let before = state.clone();
    let before_links = link_map(&before);
    let trace = step_result(state, event).map_err(|err| err.to_string())?;
    let after_links = link_map(state);
    let changed = changed_cells(&before, state, &before_links, &after_links);
    let records = record_cells(&trace.mutations);

    for cell in &changed {
        let count = records.get(cell).copied().unwrap_or(0);
        if count != 1 {
            return Err(format!("changed cell {cell:?} had {count} records"));
        }
    }
    for (cell, count) in records {
        if count != 1 {
            return Err(format!("record cell {cell:?} had duplicate records"));
        }
        if !changed.contains(&cell) {
            return Err(format!("record without final change: {cell:?}"));
        }
    }
    Ok(())
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
        if (before - after).abs() >= 1e-6 {
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
            _ => continue,
        };
        *out.entry(cell).or_insert(0) += 1;
    }
    out
}

pub fn assert_event(id: i64, label: &str, targets: &[(&str, f32)]) -> Event {
    let mut map = BTreeMap::new();
    for (axis, value) in targets {
        map.insert((*axis).to_string(), *value);
    }
    Event {
        id,
        cues: Vec::new(),
        asserts: vec![Assert {
            concept: ConceptRef::Label(label.to_string()),
            targets: map,
            weight: 1.0,
        }],
        links: Vec::new(),
        goal_ops: Vec::new(),
    }
}

pub fn cue_event(id: i64, label: &str, strength: f32) -> Event {
    Event {
        id,
        cues: vec![Cue {
            concept: ConceptRef::Label(label.to_string()),
            strength,
        }],
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: Vec::new(),
    }
}

pub fn cue_two_event(id: i64, left: &str, right: &str, strength: f32) -> Event {
    Event {
        id,
        cues: vec![
            Cue {
                concept: ConceptRef::Label(left.to_string()),
                strength,
            },
            Cue {
                concept: ConceptRef::Label(right.to_string()),
                strength,
            },
        ],
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: Vec::new(),
    }
}

pub fn link_event(id: i64, left: &str, right: &str, hint: f32) -> Event {
    Event {
        id,
        cues: Vec::new(),
        asserts: Vec::new(),
        links: vec![LinkAssert {
            left: ConceptRef::Label(left.to_string()),
            right: ConceptRef::Label(right.to_string()),
            hint,
        }],
        goal_ops: Vec::new(),
    }
}

pub fn row(state: &AmState, label: &str) -> usize {
    state
        .resolve_existing(&ConceptRef::Label(label.to_string()))
        .unwrap()
}

pub fn axis(name: &str) -> usize {
    axis_index(name).unwrap()
}
