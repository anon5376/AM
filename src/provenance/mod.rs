use crate::core::trace::{Cause, MutationTarget, StepTrace};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn provenance_report(trace_path: impl AsRef<Path>, event_id: i64) -> Result<String> {
    let text = fs::read_to_string(trace_path.as_ref())
        .with_context(|| format!("read trace {}", trace_path.as_ref().display()))?;
    let mut matches = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let trace: StepTrace =
            serde_json::from_str(line).with_context(|| format!("parse trace line {}", idx + 1))?;
        if trace.event_id == event_id {
            matches.push(trace);
        }
    }
    anyhow::ensure!(
        !matches.is_empty(),
        "event {event_id} not found in {}",
        trace_path.as_ref().display()
    );

    let mut cause_counts = BTreeMap::<Cause, usize>::new();
    let mut target_counts = BTreeMap::<MutationTarget, usize>::new();
    let mut mutation_count = 0_usize;
    let mut opened = 0_usize;
    let mut closed = 0_usize;
    let ticks = matches
        .iter()
        .map(|trace| trace.tick.to_string())
        .collect::<Vec<_>>()
        .join(",");
    for trace in &matches {
        mutation_count += trace.mutations.len();
        opened += trace.opened_contradictions.len();
        closed += trace.closed_contradictions.len();
        for mutation in &trace.mutations {
            *cause_counts.entry(mutation.cause).or_insert(0) += 1;
            *target_counts.entry(mutation.target).or_insert(0) += 1;
        }
    }

    let mut out = String::new();
    out.push_str(&format!("event {event_id}\n"));
    out.push_str(&format!("trace_file {}\n", trace_path.as_ref().display()));
    out.push_str(&format!("traces {}\n", matches.len()));
    out.push_str(&format!("ticks {ticks}\n"));
    out.push_str(&format!("mutations {mutation_count}\n"));
    out.push_str(&format!("opened_contradictions {opened}\n"));
    out.push_str(&format!("closed_contradictions {closed}\n"));
    out.push_str("causes:\n");
    append_counts(&mut out, cause_counts);
    out.push_str("targets:\n");
    append_counts(&mut out, target_counts);
    Ok(out)
}

fn append_counts<T: Ord + std::fmt::Debug>(out: &mut String, counts: BTreeMap<T, usize>) {
    if counts.is_empty() {
        out.push_str("  (none)\n");
        return;
    }
    for (key, count) in counts {
        out.push_str(&format!("  {:?} x{}\n", key, count));
    }
}
