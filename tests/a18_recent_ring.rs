use am001::beval::compile::compile_context;
use am001::core::state::{AmState, RECENT_MUTATION_CAUSE_CAPACITY};
use am001::core::theta::Theta;
use am001::core::trace::{Cause, MutationRecord, MutationTarget, StepTrace};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn recent_ring_capacity_overflow_and_event_ids_are_pinned() {
    let mut state = AmState::new(Theta::default()).unwrap();
    for event_id in 1..=25 {
        state.record_recent_mutation_causes(&single_mutation_trace(
            event_id,
            if event_id % 2 == 0 {
                Cause::Cue
            } else {
                Cause::Write
            },
        ));
        assert!(state.recent_mutation_causes.len() <= RECENT_MUTATION_CAUSE_CAPACITY);
    }

    assert_eq!(
        state.recent_mutation_causes.len(),
        RECENT_MUTATION_CAUSE_CAPACITY
    );
    assert_eq!(state.recent_mutation_causes.first().unwrap().event_id, 6);
    assert_eq!(state.recent_mutation_causes.last().unwrap().event_id, 25);
    assert_eq!(
        state
            .recent_mutation_causes
            .iter()
            .map(|entry| entry.event_id)
            .collect::<BTreeSet<_>>()
            .len(),
        RECENT_MUTATION_CAUSE_CAPACITY
    );
}

#[test]
fn recent_context_dedup_counts_without_erasing_ring_joinability() {
    let mut state = AmState::new(Theta::default()).unwrap();
    state.record_recent_mutation_causes(&single_mutation_trace(11, Cause::Cue));
    state.record_recent_mutation_causes(&single_mutation_trace(12, Cause::Cue));
    state.record_recent_mutation_causes(&single_mutation_trace(13, Cause::Write));

    let context = compile_context(&state, 1_200).unwrap();
    assert!(context.contains("Cue x2"));
    assert!(context.contains("Write x1"));
    assert_eq!(
        state
            .recent_mutation_causes
            .iter()
            .map(|entry| entry.event_id)
            .collect::<Vec<_>>(),
        vec![11, 12, 13]
    );
}

#[test]
fn dynamics_sources_do_not_read_recent_ring() {
    for path in dynamics_guard_paths() {
        let text = fs::read_to_string(&path).unwrap();
        for forbidden in [
            "recent_mutation_causes",
            "RecentMutationCause",
            "RECENT_MUTATION_CAUSE_CAPACITY",
        ] {
            assert!(
                !text.contains(forbidden),
                "A18 RECENT ring token `{forbidden}` leaked into dynamics path {}",
                path.display()
            );
        }
    }
}

fn single_mutation_trace(event_id: i64, cause: Cause) -> StepTrace {
    let mut trace = StepTrace::new(event_id, event_id);
    trace.mutations.push(MutationRecord {
        tick: event_id,
        event_id,
        target: MutationTarget::A,
        row: 0,
        axis: None,
        target_row: None,
        before: 0.0,
        after: 1.0,
        delta: 1.0,
        cause,
    });
    trace
}

fn dynamics_guard_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("src/core/settle.rs"),
        PathBuf::from("src/core/write.rs"),
        PathBuf::from("src/core/hebb.rs"),
        PathBuf::from("src/core/contradiction.rs"),
        PathBuf::from("src/core/decay.rs"),
        PathBuf::from("src/core/resolve.rs"),
    ];
    paths.extend(collect_rs(Path::new("src/world")));
    paths.sort();
    paths
}

fn collect_rs(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(collect_rs(&path));
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}
