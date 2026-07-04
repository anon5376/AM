use crate::apply::parse_batch_text;
use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngestKind {
    CargoTest,
    SweepCsv,
    WorldRun,
}

impl IngestKind {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "cargo-test" => Ok(Self::CargoTest),
            "sweep-csv" => Ok(Self::SweepCsv),
            "world-run" => Ok(Self::WorldRun),
            _ => anyhow::bail!(
                "unknown ingest kind `{value}`; expected cargo-test, sweep-csv, or world-run"
            ),
        }
    }
}

pub fn ingest_file(
    kind: IngestKind,
    input: impl AsRef<Path>,
    session_id: &str,
    events_out: impl AsRef<Path>,
) -> Result<String> {
    let text = fs::read_to_string(input.as_ref())
        .with_context(|| format!("read ingest input {}", input.as_ref().display()))?;
    let events = match kind {
        IngestKind::CargoTest => cargo_test_events(&text, session_id)?,
        IngestKind::SweepCsv => sweep_csv_events(&text, session_id)?,
        IngestKind::WorldRun => world_run_events(&text, session_id)?,
    };
    write_validated_events(events, events_out)
}

pub fn distill_file(
    input: impl AsRef<Path>,
    session_id: &str,
    events_out: impl AsRef<Path>,
) -> Result<String> {
    let text = fs::read_to_string(input.as_ref())
        .with_context(|| format!("read distill input {}", input.as_ref().display()))?;
    let block = extract_one_eg1_block(&text)?;
    let mut events = Vec::new();
    for (idx, line) in block.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut value: Value =
            serde_json::from_str(line).with_context(|| format!("parse eg1 line {}", idx + 1))?;
        force_bridge_fields(&mut value, session_id)?;
        events.push(value);
    }
    anyhow::ensure!(!events.is_empty(), "eg1 block contains no events");
    write_validated_events(events, events_out)
}

fn cargo_test_events(text: &str, session_id: &str) -> Result<Vec<Value>> {
    let summaries = parse_cargo_test_counts(text)?;
    summaries
        .into_iter()
        .enumerate()
        .map(|(idx, (passed, failed))| {
            let total = passed + failed;
            let pass_ratio = if total == 0 {
                0.0
            } else {
                passed as f32 / total as f32
            };
            Ok(assert_event(
                idx as u64 + 1,
                session_id,
                "test_verified",
                "test_suite",
                &[
                    (
                        "truth_assert",
                        if failed == 0 && total > 0 { 1.0 } else { -1.0 },
                    ),
                    ("completion", pass_ratio.clamp(-1.0, 1.0)),
                    (
                        "risk",
                        (failed as f32 / total.max(1) as f32).clamp(-1.0, 1.0),
                    ),
                    ("confidence_proxy", pass_ratio.clamp(-1.0, 1.0)),
                ],
            ))
        })
        .collect()
}

fn sweep_csv_events(text: &str, session_id: &str) -> Result<Vec<Value>> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().context("sweep csv is empty")?;
    let headers = split_csv_line(header);
    let hash_idx = headers
        .iter()
        .position(|name| *name == "theta_hash" || *name == "hash");
    let all_pass_idx = headers.iter().position(|name| *name == "all_pass");
    let recall_idx = headers
        .iter()
        .position(|name| *name == "completion_recall" || *name == "recall");
    let contamination_idx = headers
        .iter()
        .position(|name| *name == "completion_contamination" || *name == "contamination");
    let margin_idx = headers
        .iter()
        .position(|name| *name == "recall_margin_095" || *name == "margin");
    let mut best: Option<SweepRow> = None;
    for line in lines {
        let fields = split_csv_line(line);
        let hash = hash_idx
            .and_then(|idx| fields.get(idx))
            .copied()
            .filter(|value| !value.is_empty())
            .context("sweep csv row missing theta_hash/hash")?;
        let row = SweepRow {
            hash: hash.to_string(),
            all_pass: all_pass_idx
                .and_then(|idx| fields.get(idx))
                .is_some_and(|value| matches!(*value, "true" | "1" | "pass")),
            recall: parse_optional_f32(&fields, recall_idx).unwrap_or(0.0),
            contamination: parse_optional_f32(&fields, contamination_idx).unwrap_or(1.0),
            margin: parse_optional_f32(&fields, margin_idx).unwrap_or(0.0),
        };
        if row.all_pass {
            best = match best {
                Some(current) if current.margin > row.margin => Some(current),
                Some(current) if current.margin == row.margin && current.recall > row.recall => {
                    Some(current)
                }
                Some(current)
                    if current.margin == row.margin
                        && current.recall == row.recall
                        && current.hash <= row.hash =>
                {
                    Some(current)
                }
                _ => Some(row),
            };
        } else if best.is_none() {
            best = Some(row);
        }
    }
    let best = best.context("sweep csv has no data rows")?;
    let hash8 = best.hash.chars().take(8).collect::<String>();
    anyhow::ensure!(
        hash8.len() == 8,
        "sweep theta hash must have at least 8 chars"
    );
    Ok(vec![assert_event(
        1,
        session_id,
        "test_verified",
        &format!("sweep_{hash8}"),
        &[
            ("truth_assert", if best.all_pass { 1.0 } else { -1.0 }),
            ("completion", best.recall.clamp(-1.0, 1.0)),
            ("risk", best.contamination.clamp(-1.0, 1.0)),
            ("confidence_proxy", best.margin.clamp(-1.0, 1.0)),
        ],
    )])
}

fn world_run_events(text: &str, session_id: &str) -> Result<Vec<Value>> {
    let nonempty = text.lines().filter(|line| !line.trim().is_empty()).count();
    anyhow::ensure!(nonempty > 0, "world-run artifact is empty");
    let completed = text.contains("termination=goal")
        || text.contains("\"termination\":\"goal\"")
        || text.contains("\"termination\":\"success\"");
    let failed = text.contains("termination=death") || text.contains("\"termination\":\"death\"");
    Ok(vec![assert_event(
        1,
        session_id,
        "test_verified",
        "world_run",
        &[
            ("truth_assert", if failed { -1.0 } else { 1.0 }),
            ("completion", if completed { 1.0 } else { -1.0 }),
            ("risk", if failed { 1.0 } else { 0.0 }),
            ("confidence_proxy", 1.0),
        ],
    )])
}

fn parse_cargo_test_counts(text: &str) -> Result<Vec<(usize, usize)>> {
    let mut out = Vec::new();
    for line in text.lines().rev() {
        if !line.contains("test result:") {
            continue;
        }
        let passed = number_before(line, "passed").unwrap_or(0);
        let failed = number_before(line, "failed").unwrap_or(0);
        out.push((passed, failed));
    }
    out.reverse();
    if !out.is_empty() {
        return Ok(out);
    }
    anyhow::bail!("cargo-test input has no `test result:` summary")
}

fn number_before(line: &str, marker: &str) -> Option<usize> {
    let idx = line.find(marker)?;
    line[..idx].split_whitespace().last()?.parse().ok()
}

fn split_csv_line(line: &str) -> Vec<&str> {
    line.split(',').map(str::trim).collect()
}

fn parse_optional_f32(fields: &[&str], idx: Option<usize>) -> Option<f32> {
    fields.get(idx?)?.parse().ok()
}

#[derive(Clone, Debug)]
struct SweepRow {
    hash: String,
    all_pass: bool,
    recall: f32,
    contamination: f32,
    margin: f32,
}

fn extract_one_eg1_block(text: &str) -> Result<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !in_block {
            if trimmed == "```eg1" {
                in_block = true;
                current.clear();
            }
            continue;
        }
        if trimmed == "```" {
            in_block = false;
            blocks.push(current.join("\n"));
            current.clear();
            continue;
        }
        current.push(line);
    }
    anyhow::ensure!(!in_block, "unterminated eg1 fence");
    anyhow::ensure!(blocks.len() == 1, "expected exactly one eg1 fenced block");
    Ok(blocks.remove(0))
}

fn force_bridge_fields(value: &mut Value, session_id: &str) -> Result<()> {
    let object = value
        .as_object_mut()
        .context("eg1 line must be a JSON object")?;
    object.insert(
        "session_id".to_string(),
        Value::String(session_id.to_string()),
    );
    object.insert("source".to_string(), Value::String("llm_claim".to_string()));
    Ok(())
}

fn write_validated_events(events: Vec<Value>, events_out: impl AsRef<Path>) -> Result<String> {
    let mut out = String::from(r#"{"grammar":"EG-1"}"#);
    out.push('\n');
    for event in events {
        out.push_str(&serde_json::to_string(&event).context("serialize EG-1 event")?);
        out.push('\n');
    }
    let parsed = parse_batch_text(&out).map_err(|rejection| {
        anyhow::anyhow!(
            "generated EG-1 rejected: {:?}: {}",
            rejection.reason,
            rejection.detail
        )
    })?;
    anyhow::ensure!(
        parsed.rejections.is_empty() && !parsed.has_structural_rejection(),
        "generated EG-1 contains event rejections"
    );
    if let Some(parent) = events_out.as_ref().parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(events_out.as_ref(), &out)
        .with_context(|| format!("write events {}", events_out.as_ref().display()))?;
    Ok(out)
}

fn assert_event(
    id: u64,
    session_id: &str,
    source: &str,
    concept: &str,
    axes: &[(&str, f32)],
) -> Value {
    json!({
        "id": id,
        "session_id": session_id,
        "source": source,
        "verb": "assert",
        "args": {
            "concept": concept,
            "axes": axes.iter().map(|(axis, value)| axis_value(axis, *value)).collect::<Vec<_>>()
        }
    })
}

fn axis_value(axis: &str, value: f32) -> Value {
    let mut object = Map::new();
    object.insert("axis".to_string(), Value::String(axis.to_string()));
    object.insert("value".to_string(), json!(value));
    Value::Object(object)
}
