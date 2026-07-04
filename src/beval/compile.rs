use crate::beval::prompt::token_count;
use crate::core::axes::axis_name;
use crate::core::inspect::axis_certainty;
use crate::core::state::{AmState, ContradictionStatus, WriteEvidence};
use crate::storage::snapshot_file::load_snapshot;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const DEFAULT_CONTEXT_BUDGET: usize = 1_200;
pub const CONTRADICTION_DIRECTIVE: &str = "OPEN CONTRADICTIONS EXIST. Do not present affected concepts as settled. Ask for a test or propose disambiguation.";

pub fn compile_context_from_snapshot(
    snapshot: impl AsRef<Path>,
    budget_tokens: usize,
) -> Result<String> {
    let state = load_snapshot(snapshot)?;
    compile_context(&state, budget_tokens)
}

pub fn write_compiled_context(
    snapshot: impl AsRef<Path>,
    budget_tokens: usize,
    out: impl AsRef<Path>,
) -> Result<String> {
    let context = compile_context_from_snapshot(snapshot, budget_tokens)?;
    fs::write(out.as_ref(), &context)
        .with_context(|| format!("write context {}", out.as_ref().display()))?;
    Ok(context)
}

pub fn compile_context(state: &AmState, budget_tokens: usize) -> Result<String> {
    let mut sections = build_sections(state);
    truncate_sections(&mut sections, budget_tokens)?;
    let rendered = render_sections(&sections);
    anyhow::ensure!(
        token_count(&rendered) <= budget_tokens,
        "compiled context exceeds budget: {} > {} tokens",
        token_count(&rendered),
        budget_tokens
    );
    Ok(rendered)
}

fn build_sections(state: &AmState) -> Vec<Section> {
    vec![
        Some(Section::fixed(
            "STATE:",
            vec![
                format!("theta_hash {}", state.theta.hash()),
                format!("tick {}", state.tick),
                format!("row_count {}", state.allocated_rows_sorted().len()),
            ],
        )),
        Some(Section::truncatable(
            "ACTIVE:",
            active_entries(state),
            Priority::Active,
        )),
        Some(Section::truncatable(
            "LINKS:",
            link_entries(state),
            Priority::Links,
        )),
        Some(Section::fixed(
            "CONTRADICTIONS:",
            contradiction_entries(state),
        )),
        Some(Section::truncatable(
            "LOW-CERT:",
            low_cert_entries(state),
            Priority::LowCert,
        )),
        Some(Section::fixed("GOALS:", goal_entries(state))),
        Some(Section::truncatable(
            "RECENT:",
            recent_entries(state),
            Priority::Recent,
        )),
        directive_section(state),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn active_entries(state: &AmState) -> Vec<String> {
    let mut rows = state.allocated_rows_sorted();
    rows.sort_by(|left, right| {
        state.a[*right]
            .total_cmp(&state.a[*left])
            .then_with(|| left.cmp(right))
    });
    rows.into_iter()
        .take(16)
        .map(|row| {
            let mut axes = (0..state.theta.d)
                .map(|axis| (axis, state.m_get(row, axis)))
                .filter(|(_, value)| value.abs() >= state.theta.eps_log)
                .collect::<Vec<_>>();
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
                .join(" ");
            format!(
                "{} [{}] cert {:.2}",
                state.concept_label(row),
                rendered_axes,
                state.certainty(row)
            )
        })
        .collect()
}

fn link_entries(state: &AmState) -> Vec<String> {
    let mut links = BTreeMap::<(usize, usize), f32>::new();
    for source in state.allocated_rows_sorted() {
        for link in &state.links[source] {
            if !state.is_allocated(link.target) {
                continue;
            }
            let pair = if source <= link.target {
                (source, link.target)
            } else {
                (link.target, source)
            };
            links
                .entry(pair)
                .and_modify(|weight| *weight = weight.max(link.weight))
                .or_insert(link.weight);
        }
    }
    let mut entries = links.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0 .0.cmp(&right.0 .0))
            .then_with(|| left.0 .1.cmp(&right.0 .1))
    });
    entries
        .into_iter()
        .take(12)
        .map(|((left, right), weight)| {
            format!(
                "{} —{:.2}→ {}",
                state.concept_label(left),
                weight,
                state.concept_label(right)
            )
        })
        .collect()
}

fn contradiction_entries(state: &AmState) -> Vec<String> {
    let mut entries = state
        .open_contradictions
        .iter()
        .filter(|contradiction| contradiction.status == ContradictionStatus::Open)
        .filter_map(|contradiction| {
            let row = state.resolve_row_ref(contradiction.concept).ok()?.id;
            Some((
                row,
                contradiction.axis,
                contradiction.opened_tick,
                format!(
                    "{} {} (min-axis cert {:.2})",
                    state.concept_label(row),
                    axis_name(contradiction.axis).unwrap_or("?"),
                    axis_certainty(state.v_get(row, contradiction.axis))
                ),
            ))
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    entries
        .into_iter()
        .map(|(_, _, _, rendered)| rendered)
        .collect()
}

fn low_cert_entries(state: &AmState) -> Vec<String> {
    let active_rows = state
        .allocated_rows_sorted()
        .into_iter()
        .filter(|row| state.a[*row] > state.theta.eps_log)
        .collect::<Vec<_>>();
    if active_rows.is_empty() {
        return Vec::new();
    }
    let mut axes = (0..state.theta.d)
        .map(|axis| {
            let avg_v = active_rows
                .iter()
                .map(|row| state.v_get(*row, axis))
                .sum::<f32>()
                / active_rows.len() as f32;
            (axis, axis_certainty(avg_v))
        })
        .collect::<Vec<_>>();
    axes.sort_by(|left, right| {
        left.1
            .total_cmp(&right.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    axes.into_iter()
        .take(8)
        .map(|(axis, cert)| format!("{} cert {:.2}", axis_name(axis).unwrap_or("?"), cert))
        .collect()
}

fn goal_entries(state: &AmState) -> Vec<String> {
    state
        .goals
        .iter()
        .filter_map(|goal| state.resolve_row_ref(*goal).ok())
        .map(|goal| state.concept_label(goal.id))
        .collect()
}

fn recent_entries(state: &AmState) -> Vec<String> {
    let mut writes = state
        .recent_writes
        .iter()
        .flat_map(|((_, _), history)| history.iter())
        .collect::<Vec<&WriteEvidence>>();
    writes.sort_by(|left, right| {
        right
            .tick
            .cmp(&left.tick)
            .then_with(|| right.event_id.cmp(&left.event_id))
    });
    let count = writes.into_iter().take(20).count();
    if count == 0 {
        Vec::new()
    } else {
        vec![format!("write x{count}")]
    }
}

fn directive_section(state: &AmState) -> Option<Section> {
    let has_open = state
        .open_contradictions
        .iter()
        .any(|contradiction| contradiction.status == ContradictionStatus::Open);
    has_open.then(|| Section::fixed("DIRECTIVE:", vec![CONTRADICTION_DIRECTIVE.to_string()]))
}

fn truncate_sections(sections: &mut [Section], budget_tokens: usize) -> Result<()> {
    while token_count(&render_sections(sections)) > budget_tokens {
        let mut removed = false;
        for priority in [
            Priority::Recent,
            Priority::LowCert,
            Priority::Links,
            Priority::Active,
        ] {
            if let Some(section) = sections
                .iter_mut()
                .find(|section| section.priority == Some(priority) && !section.entries.is_empty())
            {
                section.entries.pop();
                removed = true;
                break;
            }
        }
        if !removed {
            anyhow::bail!(
                "compiled context fixed sections exceed budget {} tokens",
                budget_tokens
            );
        }
    }
    Ok(())
}

fn render_sections(sections: &[Section]) -> String {
    let mut out = String::new();
    for section in sections {
        out.push_str(section.header);
        out.push('\n');
        if section.entries.is_empty() {
            out.push_str("(none)\n");
        } else {
            for entry in &section.entries {
                out.push_str(entry);
                out.push('\n');
            }
        }
    }
    out
}

#[derive(Clone, Debug)]
struct Section {
    header: &'static str,
    entries: Vec<String>,
    priority: Option<Priority>,
}

impl Section {
    fn fixed(header: &'static str, entries: Vec<String>) -> Self {
        Self {
            header,
            entries,
            priority: None,
        }
    }

    fn truncatable(header: &'static str, entries: Vec<String>, priority: Priority) -> Self {
        Self {
            header,
            entries,
            priority: Some(priority),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Priority {
    Recent,
    LowCert,
    Links,
    Active,
}
