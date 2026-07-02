use crate::core::axes::axis_name;
use crate::core::state::{AmState, ContradictionStatus};
use crate::core::trace::{Cause, MutationRecord, MutationTarget, StepTrace};
use anyhow::{Context, Result};

pub fn dump_state(state: &AmState, sort: &str, top: usize) -> Result<String> {
    let mut rows = state.allocated_rows_sorted();
    if let Some(axis) = sort.strip_prefix("axis:") {
        let axis_idx =
            crate::core::axes::axis_index(axis).with_context(|| format!("unknown axis {axis}"))?;
        rows.sort_by(|left, right| {
            state
                .m_get(*right, axis_idx)
                .total_cmp(&state.m_get(*left, axis_idx))
                .then_with(|| left.cmp(right))
        });
    } else {
        match sort {
            "act" => rows.sort_by(|left, right| {
                state.a[*right]
                    .total_cmp(&state.a[*left])
                    .then_with(|| left.cmp(right))
            }),
            "b" => rows.sort_by(|left, right| {
                state.b[*right]
                    .total_cmp(&state.b[*left])
                    .then_with(|| left.cmp(right))
            }),
            "id" => {}
            other => anyhow::bail!("unknown sort {other}"),
        }
    }

    let mut out = String::new();
    for row in rows.into_iter().take(top) {
        let mut axes: Vec<(usize, f32)> = (0..state.theta.d)
            .map(|axis| (axis, state.m_get(row, axis)))
            .filter(|(_, value)| value.abs() >= state.theta.eps_log)
            .collect();
        axes.sort_by(|left, right| {
            right
                .1
                .abs()
                .total_cmp(&left.1.abs())
                .then_with(|| left.0.cmp(&right.0))
        });
        let rendered_axes = axes
            .into_iter()
            .take(4)
            .map(|(axis, value)| format!("{} {:+.2}", axis_name(axis).unwrap_or("?"), value))
            .collect::<Vec<_>>()
            .join("  ");
        let age = state.tick - state.u[row];
        out.push_str(&format!(
            "{:<12} [{}]  a={:.2} b={:.2} cert={:.2} age={}",
            state.concept_label(row),
            rendered_axes,
            state.a[row],
            state.b[row],
            state.certainty(row),
            age
        ));
        if let Some((axis, cert)) = min_axis_certainty_note(state, row) {
            out.push_str(&format!(
                "  (min-axis cert: {} {:.2})",
                axis_name(axis).unwrap_or("?"),
                cert
            ));
        }
        out.push('\n');
        for contradiction in state.open_contradictions.iter().filter(|contradiction| {
            contradiction.concept == state.row_ref(row)
                && contradiction.status == ContradictionStatus::Open
        }) {
            out.push_str(&format!(
                "   ⚠ contradiction open: {}\n",
                axis_name(contradiction.axis).unwrap_or("?")
            ));
        }
    }
    Ok(out)
}

pub fn axes_report(state: &AmState, label: &str) -> Result<String> {
    let row = state.resolve_existing(&crate::core::event::ConceptRef::Label(label.to_string()))?;
    let mut out = String::new();
    out.push_str(&format!(
        "{}  row={} gen={} cert={:.3}\n",
        state.concept_label(row),
        row,
        state.generation[row],
        state.certainty(row)
    ));
    for axis in 0..state.theta.d {
        let v = state.v_get(row, axis);
        out.push_str(&format!(
            "{:<28} value={:+.4} V={:.4} cert={:.4}\n",
            axis_name(axis).unwrap_or("?"),
            state.m_get(row, axis),
            v,
            axis_certainty(v)
        ));
    }
    Ok(out)
}

pub fn axis_certainty(v: f32) -> f32 {
    1.0 / (1.0 + v)
}

fn min_axis_certainty_note(state: &AmState, row: usize) -> Option<(usize, f32)> {
    let mut candidates = Vec::new();
    for axis in 0..state.theta.d {
        if state.v_get(row, axis) > state.theta.th_v {
            candidates.push(axis);
        }
    }
    for contradiction in state.open_contradictions.iter().filter(|contradiction| {
        contradiction.concept == state.row_ref(row)
            && contradiction.status == ContradictionStatus::Open
    }) {
        if !candidates.contains(&contradiction.axis) {
            candidates.push(contradiction.axis);
        }
    }
    candidates
        .into_iter()
        .map(|axis| (axis, axis_certainty(state.v_get(row, axis))))
        .min_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then_with(|| left.0.cmp(&right.0))
        })
}

pub fn format_diff(state: &AmState, trace: &StepTrace) -> String {
    let mut out = String::new();
    for mutation in &trace.mutations {
        out.push_str(&format!("{}\n", format_mutation(state, mutation)));
    }
    out
}

fn format_mutation(state: &AmState, mutation: &MutationRecord) -> String {
    let label = state.concept_label(mutation.row);
    let cause = cause_text(&mutation.cause);
    match mutation.target {
        MutationTarget::M => format!(
            "M[{}][{}]  {:+.3}   ← {}",
            label,
            axis_name(mutation.axis.unwrap_or(0)).unwrap_or("?"),
            mutation.delta,
            cause
        ),
        MutationTarget::V => format!(
            "V[{}][{}]  {:+.3}   ← {}",
            label,
            axis_name(mutation.axis.unwrap_or(0)).unwrap_or("?"),
            mutation.delta,
            cause
        ),
        MutationTarget::A => format!(
            "a[{}]  {:.3}→{:.3}   ← {}",
            label, mutation.before, mutation.after, cause
        ),
        MutationTarget::B => format!("b[{}]  {:+.3}   ← {}", label, mutation.delta, cause),
        MutationTarget::U => format!(
            "u[{}]  {:.0}→{:.0}   ← {}",
            label, mutation.before, mutation.after, cause
        ),
        MutationTarget::W => {
            let target = mutation
                .target_row
                .map(|row| state.concept_label(row))
                .unwrap_or_else(|| "?".to_string());
            format!(
                "W[{}↔{}]  {:+.3}   ← {}",
                label, target, mutation.delta, cause
            )
        }
        MutationTarget::Generation => format!(
            "generation[{}]  {:.0}→{:.0}   ← {}",
            label, mutation.before, mutation.after, cause
        ),
        MutationTarget::Allocation => format!("alloc[{}]   ← {}", label, cause),
        MutationTarget::Label => format!("label[{}]   ← {}", label, cause),
        MutationTarget::Goal => format!("goal[{}]   ← {}", label, cause),
        MutationTarget::Alias => format!(
            "alias[{}→{}]   ← {}",
            label,
            mutation
                .target_row
                .map(|row| state.concept_label(row))
                .unwrap_or_else(|| "?".to_string()),
            cause
        ),
        MutationTarget::Free => format!("free[{}]   ← {}", label, cause),
        MutationTarget::Contradiction => format!(
            "contradiction[{}][{}]   ← {}",
            label,
            mutation.target_row.and_then(axis_name).unwrap_or("?"),
            cause
        ),
    }
}

fn cause_text(cause: &Cause) -> &'static str {
    match cause {
        Cause::Allocate => "allocate",
        Cause::Cue => "cue",
        Cause::AssertActivate => "assert activation",
        Cause::Settle => "settle",
        Cause::Write => "write",
        Cause::VarianceUpdate => "variance update",
        Cause::Hebb => "hebb",
        Cause::LinkHint => "link hint",
        Cause::LinkDecay => "link decay",
        Cause::LinkPrune => "link prune",
        Cause::ContradictionOpen => "contradiction_open",
        Cause::ContradictionClose => "contradiction_close",
        Cause::ActivationDecay => "decay lambda_a",
        Cause::BaselineDecay => "baseline lambda_b",
        Cause::GarbageCollect => "garbage collect",
        Cause::Merge => "merge",
        Cause::GoalPush => "goal push",
        Cause::GoalPop => "goal pop",
        Cause::FreeRow => "free row",
        Cause::RowGeneration => "row generation",
    }
}
