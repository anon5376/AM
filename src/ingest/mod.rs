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
        reject_non_opaque_labels(&value)?;
        events.push(value);
    }
    anyhow::ensure!(!events.is_empty(), "eg1 block contains no events");
    write_validated_events(events, events_out)
}

fn cargo_test_events(text: &str, session_id: &str) -> Result<Vec<Value>> {
    let (passed, failed) = parse_cargo_test_counts(text)?;
    let total = passed + failed;
    let success = total > 0 && failed == 0;
    let confidence = if total == 0 {
        0.0
    } else {
        passed as f32 / total as f32
    };
    Ok(vec![assert_event(
        1,
        session_id,
        "test_verified",
        "cpt_1",
        &[
            ("truth_assert", if success { 1.0 } else { -1.0 }),
            ("completion", if success { 1.0 } else { -1.0 }),
            ("confidence_proxy", confidence),
        ],
    )])
}

fn sweep_csv_events(text: &str, session_id: &str) -> Result<Vec<Value>> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().context("sweep csv is empty")?;
    let headers = split_csv_line(header);
    let all_pass_idx = headers.iter().position(|name| *name == "all_pass");
    let recall_idx = headers
        .iter()
        .position(|name| *name == "completion_recall" || *name == "recall");
    let contamination_idx = headers
        .iter()
        .position(|name| *name == "completion_contamination" || *name == "contamination");
    let mut rows = 0_usize;
    let mut pass_rows = 0_usize;
    let mut best_recall = 0.0_f32;
    let mut best_contamination = 1.0_f32;
    for line in lines {
        let fields = split_csv_line(line);
        rows += 1;
        if let Some(idx) = all_pass_idx {
            if fields
                .get(idx)
                .is_some_and(|value| matches!(*value, "true" | "1" | "pass"))
            {
                pass_rows += 1;
            }
        }
        if let Some(idx) = recall_idx {
            if let Some(value) = fields.get(idx).and_then(|value| value.parse::<f32>().ok()) {
                best_recall = best_recall.max(value);
            }
        }
        if let Some(idx) = contamination_idx {
            if let Some(value) = fields.get(idx).and_then(|value| value.parse::<f32>().ok()) {
                best_contamination = best_contamination.min(value);
            }
        }
    }
    anyhow::ensure!(rows > 0, "sweep csv has no data rows");
    let pass_ratio = pass_rows as f32 / rows as f32;
    Ok(vec![assert_event(
        1,
        session_id,
        "test_verified",
        "cpt_2",
        &[
            ("truth_assert", if pass_rows > 0 { 1.0 } else { -1.0 }),
            ("completion", best_recall.max(pass_ratio).clamp(0.0, 1.0)),
            ("risk", best_contamination.clamp(0.0, 1.0)),
            ("confidence_proxy", pass_ratio.clamp(0.0, 1.0)),
        ],
    )])
}

fn world_run_events(text: &str, session_id: &str) -> Result<Vec<Value>> {
    let nonempty = text.lines().filter(|line| !line.trim().is_empty()).count();
    anyhow::ensure!(nonempty > 0, "world-run artifact is empty");
    let completed = text.contains("termination=goal")
        || text.contains("\"termination\":\"goal\"")
        || text.contains("\"termination\":\"success\"");
    Ok(vec![assert_event(
        1,
        session_id,
        "test_verified",
        "cpt_3",
        &[
            ("truth_assert", 1.0),
            ("completion", if completed { 1.0 } else { 0.5 }),
            ("confidence_proxy", 1.0),
        ],
    )])
}

fn parse_cargo_test_counts(text: &str) -> Result<(usize, usize)> {
    for line in text.lines().rev() {
        if !line.contains("test result:") {
            continue;
        }
        let passed = number_before(line, "passed").unwrap_or(0);
        let failed = number_before(line, "failed").unwrap_or(0);
        return Ok((passed, failed));
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

fn reject_non_opaque_labels(value: &Value) -> Result<()> {
    let object = value
        .as_object()
        .context("eg1 line must be a JSON object")?;
    let Some(args) = object.get("args").and_then(Value::as_object) else {
        return Ok(());
    };
    for field in ["concept", "a", "b"] {
        if let Some(label) = args.get(field).and_then(Value::as_str) {
            anyhow::ensure!(
                is_opaque_label(label),
                "distill label `{label}` is not opaque"
            );
        }
    }
    Ok(())
}

fn is_opaque_label(label: &str) -> bool {
    for prefix in ["cpt_", "trk_", "loc_"] {
        if let Some(rest) = label.strip_prefix(prefix) {
            return !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit());
        }
    }
    false
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
