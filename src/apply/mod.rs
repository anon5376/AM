use crate::core::axes::axis_index;
use crate::core::diff::trace_jsonl_bytes;
use crate::core::event::{Assert, ConceptRef, Cue, Event, GoalOp, LinkAssert};
use crate::core::state::AmState;
use crate::core::step::step_result;
use crate::core::trace::StepTrace;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const EG1_VERSION: &str = "EG-1";
pub const STAGING_FORMAT_VERSION: u32 = 1;
pub const STAGING_EXPIRY_APPLIED_EVENTS: u64 = 200;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    User,
    TestVerified,
    LlmClaim,
}

impl EventSource {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Self::User),
            "test_verified" => Some(Self::TestVerified),
            "llm_claim" => Some(Self::LlmClaim),
            _ => None,
        }
    }

    fn is_non_staged(self) -> bool {
        matches!(self, Self::User | Self::TestVerified)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EgVerb {
    Assert,
    Cue,
    Link,
    GoalPush,
    GoalPop,
}

impl EgVerb {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "assert" => Some(Self::Assert),
            "cue" => Some(Self::Cue),
            "link" => Some(Self::Link),
            "goal_push" => Some(Self::GoalPush),
            "goal_pop" => Some(Self::GoalPop),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EgArgs {
    Assert {
        concept: String,
        axes: Vec<EgAxisValue>,
    },
    Cue {
        concept: String,
        strength: f32,
    },
    Link {
        a: String,
        b: String,
        weight: f32,
    },
    Goal {
        concept: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EgAxisValue {
    pub axis: String,
    pub value: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EgEnvelope {
    pub id: u64,
    pub session_id: String,
    pub source: EventSource,
    pub verb: EgVerb,
    pub args: EgArgs,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RejectReason {
    BadVersion,
    MissingHeader,
    BadJson,
    UnknownField,
    MissingField,
    UnknownVerb,
    UnknownSource,
    NonMonotonicId,
    IdOutOfRange,
    EmptySessionId,
    BadArgs,
    NonFiniteValue,
    UnknownAxis,
    StrengthOutOfRange,
    WeightOutOfRange,
    CueClaimRejected,
    UnimplementedVerb,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplyAction {
    Applied,
    Staged,
    Rejected,
    CommittedFromStaging,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventVerdict {
    pub id: Option<u64>,
    pub action: ApplyAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<EventSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<RejectReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committed_by: Option<u64>,
}

impl EventVerdict {
    fn rejected(id: Option<u64>, reason: RejectReason, detail: impl Into<String>) -> Self {
        Self {
            id,
            action: ApplyAction::Rejected,
            source: None,
            reason: Some(reason),
            detail: Some(detail.into()),
            committed_by: None,
        }
    }

    fn for_event(event: &EgEnvelope, action: ApplyAction) -> Self {
        Self {
            id: Some(event.id),
            action,
            source: Some(event.source),
            reason: None,
            detail: None,
            committed_by: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ApplySummary {
    pub applied: usize,
    pub staged: usize,
    pub rejected: usize,
    pub committed_from_staging: usize,
    pub expired: usize,
    pub structural_rejections: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApplyReport {
    pub grammar: String,
    pub summary: ApplySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staging_file: Option<String>,
    pub events: Vec<EventVerdict>,
}

impl ApplyReport {
    pub fn from_verdicts(verdicts: Vec<EventVerdict>, structural_rejections: usize) -> Self {
        let mut summary = ApplySummary {
            structural_rejections,
            ..ApplySummary::default()
        };
        for verdict in &verdicts {
            match verdict.action {
                ApplyAction::Applied => summary.applied += 1,
                ApplyAction::Staged => summary.staged += 1,
                ApplyAction::Rejected => summary.rejected += 1,
                ApplyAction::CommittedFromStaging => summary.committed_from_staging += 1,
                ApplyAction::Expired => summary.expired += 1,
            }
        }
        Self {
            grammar: EG1_VERSION.to_string(),
            summary,
            trace_file: None,
            staging_file: None,
            events: verdicts,
        }
    }

    pub fn has_rejections(&self) -> bool {
        self.summary.rejected > 0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedBatch {
    pub events: Vec<EgEnvelope>,
    pub rejections: Vec<EventVerdict>,
    pub structural_rejections: usize,
}

impl ParsedBatch {
    pub fn has_structural_rejection(&self) -> bool {
        self.structural_rejections > 0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderRejection {
    pub reason: RejectReason,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StagedEntry {
    pub event: EgEnvelope,
    pub staged_at_event_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StagingTombstone {
    pub event: EgEnvelope,
    pub staged_at_event_count: u64,
    pub expired_at_event_count: u64,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StagingSidecar {
    pub format_version: u32,
    pub grammar: String,
    pub applied_event_count: u64,
    pub entries: Vec<StagedEntry>,
    pub tombstones: Vec<StagingTombstone>,
}

impl Default for StagingSidecar {
    fn default() -> Self {
        Self {
            format_version: STAGING_FORMAT_VERSION,
            grammar: EG1_VERSION.to_string(),
            applied_event_count: 0,
            entries: Vec::new(),
            tombstones: Vec::new(),
        }
    }
}

impl StagingSidecar {
    pub fn normalize(&mut self) {
        self.entries.sort_by_key(staged_key);
        self.tombstones.sort_by_key(|tombstone| {
            (
                staged_key_from_parts(&tombstone.event),
                tombstone.expired_at_event_count,
            )
        });
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut clone = self.clone();
        clone.normalize();
        serde_json::to_vec(&clone).context("serialize staging sidecar")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut sidecar: Self = serde_json::from_slice(bytes).context("staging sidecar corrupt")?;
        anyhow::ensure!(
            sidecar.format_version == STAGING_FORMAT_VERSION,
            "staging sidecar format_version {} is incompatible with expected {}",
            sidecar.format_version,
            STAGING_FORMAT_VERSION
        );
        anyhow::ensure!(
            sidecar.grammar == EG1_VERSION,
            "staging sidecar grammar {} is incompatible with expected {}",
            sidecar.grammar,
            EG1_VERSION
        );
        sidecar.normalize();
        Ok(sidecar)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApplyOutcome {
    pub report: ApplyReport,
    pub traces: Vec<StepTrace>,
}

pub fn staging_path_for(snapshot_path: &Path) -> PathBuf {
    sibling_with_added_suffix(snapshot_path, ".staging")
}

pub fn trace_path_for(snapshot_path: &Path) -> PathBuf {
    sibling_with_added_suffix(snapshot_path, ".trace.jsonl")
}

pub fn load_staging(path: &Path) -> Result<StagingSidecar> {
    if !path.exists() {
        return Ok(StagingSidecar::default());
    }
    let bytes = fs::read(path).with_context(|| format!("read staging {}", path.display()))?;
    StagingSidecar::from_bytes(&bytes)
}

pub fn save_staging(path: &Path, sidecar: &StagingSidecar) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, sidecar.to_bytes()?)
        .with_context(|| format!("write staging {}", path.display()))
}

pub fn write_trace_file(path: &Path, traces: &[StepTrace]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, trace_jsonl_bytes(traces)?)
        .with_context(|| format!("write apply trace {}", path.display()))
}

pub fn write_report(path: &Path, report: &ApplyReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(report).context("serialize apply report")?;
    fs::write(path, bytes).with_context(|| format!("write apply report {}", path.display()))
}

pub fn parse_batch_text(text: &str) -> std::result::Result<ParsedBatch, HeaderRejection> {
    let mut lines = text.lines().enumerate();
    let Some((_, header_line)) = lines.next() else {
        return Err(HeaderRejection {
            reason: RejectReason::MissingHeader,
            detail: "EG-1 file is empty".to_string(),
        });
    };
    parse_header(header_line)?;

    let mut events = Vec::new();
    let mut rejections = Vec::new();
    let mut structural_rejections = 0;
    let mut last_id = None;
    for (line_index, line) in lines {
        if line.trim().is_empty() {
            continue;
        }
        match parse_envelope_line(line, line_index + 1, last_id) {
            Ok(event) => {
                last_id = Some(event.id);
                events.push(event);
            }
            Err(parsed) => {
                if parsed.structural {
                    structural_rejections += 1;
                }
                if let Some(id) = parsed.id {
                    if last_id.map(|last| id > last).unwrap_or(true) {
                        last_id = Some(id);
                    }
                }
                rejections.push(EventVerdict::rejected(
                    parsed.id,
                    parsed.reason,
                    parsed.detail,
                ));
            }
        }
    }

    Ok(ParsedBatch {
        events,
        rejections,
        structural_rejections,
    })
}

pub fn header_rejection_report(rejection: HeaderRejection) -> ApplyReport {
    ApplyReport::from_verdicts(
        vec![EventVerdict::rejected(
            None,
            rejection.reason,
            rejection.detail,
        )],
        1,
    )
}

pub fn apply_parsed_batch(
    state: &mut AmState,
    batch: &ParsedBatch,
    staging: &mut StagingSidecar,
) -> Result<ApplyOutcome> {
    if batch.has_structural_rejection() {
        return Ok(ApplyOutcome {
            report: ApplyReport::from_verdicts(
                batch.rejections.clone(),
                batch.structural_rejections,
            ),
            traces: Vec::new(),
        });
    }

    let mut verdicts = Vec::new();
    let mut pending_rejections = batch.rejections.clone();
    pending_rejections.sort_by_key(|verdict| verdict.id.unwrap_or(u64::MAX));
    let mut traces = Vec::new();
    for event in &batch.events {
        emit_rejections_before(event.id, &mut pending_rejections, &mut verdicts);
        match event.source {
            EventSource::User | EventSource::TestVerified => {
                let core_event = event.to_core_event()?;
                let trace = step_result(state, &core_event)?;
                traces.push(trace);
                staging.applied_event_count += 1;
                verdicts.push(EventVerdict::for_event(event, ApplyAction::Applied));
                commit_matching_staged(state, staging, event, &mut traces, &mut verdicts)?;
                expire_staged(staging, &mut verdicts);
            }
            EventSource::LlmClaim => match event.verb {
                EgVerb::Cue => verdicts.push(EventVerdict {
                    id: Some(event.id),
                    action: ApplyAction::Rejected,
                    source: Some(event.source),
                    reason: Some(RejectReason::CueClaimRejected),
                    detail: Some(
                        "llm_claim cue events are attention steering and are not staged"
                            .to_string(),
                    ),
                    committed_by: None,
                }),
                EgVerb::GoalPush | EgVerb::GoalPop => verdicts.push(EventVerdict {
                    id: Some(event.id),
                    action: ApplyAction::Rejected,
                    source: Some(event.source),
                    reason: Some(RejectReason::UnimplementedVerb),
                    detail: Some(
                        "llm_claim goal events have no B01 corroboration rule".to_string(),
                    ),
                    committed_by: None,
                }),
                EgVerb::Assert | EgVerb::Link => {
                    staging
                        .entries
                        .extend(staged_entries_for(event, staging.applied_event_count));
                    staging.normalize();
                    verdicts.push(EventVerdict::for_event(event, ApplyAction::Staged));
                }
            },
        }
    }
    verdicts.extend(pending_rejections);

    staging.normalize();
    Ok(ApplyOutcome {
        report: ApplyReport::from_verdicts(verdicts, batch.structural_rejections),
        traces,
    })
}

fn emit_rejections_before(
    event_id: u64,
    pending_rejections: &mut Vec<EventVerdict>,
    verdicts: &mut Vec<EventVerdict>,
) {
    let split_at = pending_rejections
        .partition_point(|verdict| verdict.id.map(|id| id < event_id).unwrap_or(false));
    if split_at > 0 {
        verdicts.extend(pending_rejections.drain(..split_at));
    }
}

impl EgEnvelope {
    pub fn to_core_event(&self) -> Result<Event> {
        let id = i64::try_from(self.id).context("EG-1 id exceeds i64 trace range")?;
        let mut event = match &self.args {
            EgArgs::Assert { concept, axes } => {
                let mut targets = BTreeMap::new();
                for axis in axes {
                    targets.insert(axis.axis.clone(), axis.value);
                }
                Event {
                    id,
                    cues: Vec::new(),
                    asserts: vec![Assert {
                        concept: ConceptRef::Label(concept.clone()),
                        targets,
                        weight: 1.0,
                    }],
                    links: Vec::new(),
                    goal_ops: Vec::new(),
                }
            }
            EgArgs::Cue { concept, strength } => Event {
                id,
                cues: vec![Cue {
                    concept: ConceptRef::Label(concept.clone()),
                    strength: *strength,
                }],
                asserts: Vec::new(),
                links: Vec::new(),
                goal_ops: Vec::new(),
            },
            EgArgs::Link { a, b, weight } => Event {
                id,
                cues: Vec::new(),
                asserts: Vec::new(),
                links: vec![LinkAssert {
                    left: ConceptRef::Label(a.clone()),
                    right: ConceptRef::Label(b.clone()),
                    hint: *weight,
                }],
                goal_ops: Vec::new(),
            },
            EgArgs::Goal { concept } => {
                let op = match self.verb {
                    EgVerb::GoalPush => GoalOp::Push(ConceptRef::Label(concept.clone())),
                    EgVerb::GoalPop => GoalOp::Pop(ConceptRef::Label(concept.clone())),
                    _ => anyhow::bail!("goal args used with non-goal verb"),
                };
                Event {
                    id,
                    cues: Vec::new(),
                    asserts: Vec::new(),
                    links: Vec::new(),
                    goal_ops: vec![op],
                }
            }
        };
        event.validate_and_clamp()?;
        Ok(event)
    }
}

fn parse_header(line: &str) -> std::result::Result<(), HeaderRejection> {
    let value: Value = serde_json::from_str(line).map_err(|err| HeaderRejection {
        reason: RejectReason::BadJson,
        detail: format!("header is not JSON: {err}"),
    })?;
    let object = value.as_object().ok_or_else(|| HeaderRejection {
        reason: RejectReason::BadVersion,
        detail: "header must be a JSON object".to_string(),
    })?;
    let allowed = BTreeSet::from(["grammar"]);
    if let Some(field) = first_unknown_field(object, &allowed) {
        return Err(HeaderRejection {
            reason: RejectReason::UnknownField,
            detail: format!("unknown header field {field}"),
        });
    }
    let Some(grammar) = object.get("grammar").and_then(Value::as_str) else {
        return Err(HeaderRejection {
            reason: RejectReason::BadVersion,
            detail: "header grammar must be EG-1".to_string(),
        });
    };
    if grammar != EG1_VERSION {
        return Err(HeaderRejection {
            reason: RejectReason::BadVersion,
            detail: format!("unsupported grammar {grammar}"),
        });
    }
    Ok(())
}

struct ParsedLineError {
    id: Option<u64>,
    reason: RejectReason,
    detail: String,
    structural: bool,
}

fn parse_envelope_line(
    line: &str,
    line_number: usize,
    last_id: Option<u64>,
) -> std::result::Result<EgEnvelope, ParsedLineError> {
    let value: Value = serde_json::from_str(line).map_err(|err| ParsedLineError {
        id: None,
        reason: RejectReason::BadJson,
        detail: format!("line {line_number} is not JSON: {err}"),
        structural: true,
    })?;
    let object = value.as_object().ok_or_else(|| ParsedLineError {
        id: None,
        reason: RejectReason::BadArgs,
        detail: format!("line {line_number} envelope must be a JSON object"),
        structural: true,
    })?;
    let allowed = BTreeSet::from(["id", "session_id", "source", "verb", "args"]);
    if let Some(field) = first_unknown_field(object, &allowed) {
        return Err(ParsedLineError {
            id: parse_id_lossy(object),
            reason: RejectReason::UnknownField,
            detail: format!("line {line_number} unknown envelope field {field}"),
            structural: true,
        });
    }

    let id = parse_required_u64(object, "id", line_number)?;
    if let Some(last) = last_id {
        if id <= last {
            return Err(ParsedLineError {
                id: Some(id),
                reason: RejectReason::NonMonotonicId,
                detail: format!("line {line_number} id {id} is not greater than previous {last}"),
                structural: true,
            });
        }
    }
    if id > i64::MAX as u64 {
        return Err(ParsedLineError {
            id: Some(id),
            reason: RejectReason::IdOutOfRange,
            detail: format!("line {line_number} id exceeds i64 trace range"),
            structural: false,
        });
    }

    let session_id = parse_required_string(object, "session_id", line_number)?;
    if session_id.trim().is_empty() {
        return Err(ParsedLineError {
            id: Some(id),
            reason: RejectReason::EmptySessionId,
            detail: format!("line {line_number} session_id is empty"),
            structural: false,
        });
    }
    let source_text = parse_required_string(object, "source", line_number)?;
    let source = EventSource::parse(&source_text).ok_or_else(|| ParsedLineError {
        id: Some(id),
        reason: RejectReason::UnknownSource,
        detail: format!("line {line_number} unknown source {source_text}"),
        structural: true,
    })?;
    let verb_text = parse_required_string(object, "verb", line_number)?;
    let verb = EgVerb::parse(&verb_text).ok_or_else(|| ParsedLineError {
        id: Some(id),
        reason: RejectReason::UnknownVerb,
        detail: format!("line {line_number} unknown verb {verb_text}"),
        structural: true,
    })?;
    let args_value = object.get("args").ok_or_else(|| ParsedLineError {
        id: Some(id),
        reason: RejectReason::MissingField,
        detail: format!("line {line_number} missing field args"),
        structural: true,
    })?;
    let args = parse_args(verb, args_value, line_number, id)?;

    Ok(EgEnvelope {
        id,
        session_id,
        source,
        verb,
        args,
    })
}

fn parse_args(
    verb: EgVerb,
    value: &Value,
    line_number: usize,
    id: u64,
) -> std::result::Result<EgArgs, ParsedLineError> {
    let object = value.as_object().ok_or_else(|| ParsedLineError {
        id: Some(id),
        reason: RejectReason::BadArgs,
        detail: format!("line {line_number} args must be a JSON object"),
        structural: true,
    })?;
    match verb {
        EgVerb::Assert => {
            ensure_allowed_fields(object, &["concept", "axes"], line_number, id)?;
            let concept = parse_required_string(object, "concept", line_number)?;
            let axes_value = object.get("axes").ok_or_else(|| ParsedLineError {
                id: Some(id),
                reason: RejectReason::MissingField,
                detail: format!("line {line_number} missing field axes"),
                structural: true,
            })?;
            let axes_raw = axes_value.as_array().ok_or_else(|| ParsedLineError {
                id: Some(id),
                reason: RejectReason::BadArgs,
                detail: format!("line {line_number} axes must be an array"),
                structural: true,
            })?;
            if axes_raw.is_empty() {
                return Err(ParsedLineError {
                    id: Some(id),
                    reason: RejectReason::BadArgs,
                    detail: format!("line {line_number} axes must not be empty"),
                    structural: false,
                });
            }
            let mut axes = Vec::new();
            for axis_value in axes_raw {
                let axis_object = axis_value.as_object().ok_or_else(|| ParsedLineError {
                    id: Some(id),
                    reason: RejectReason::BadArgs,
                    detail: format!("line {line_number} axis entry must be an object"),
                    structural: true,
                })?;
                ensure_allowed_fields(axis_object, &["axis", "value"], line_number, id)?;
                let axis = parse_required_string(axis_object, "axis", line_number)?;
                if axis_index(&axis).is_none() {
                    return Err(ParsedLineError {
                        id: Some(id),
                        reason: RejectReason::UnknownAxis,
                        detail: format!("line {line_number} unknown axis {axis}"),
                        structural: false,
                    });
                }
                let value = parse_required_f32(axis_object, "value", line_number, id)?;
                axes.push(EgAxisValue { axis, value });
            }
            Ok(EgArgs::Assert { concept, axes })
        }
        EgVerb::Cue => {
            ensure_allowed_fields(object, &["concept", "strength"], line_number, id)?;
            let concept = parse_required_string(object, "concept", line_number)?;
            let strength = parse_required_f32(object, "strength", line_number, id)?;
            if !(strength > 0.0 && strength <= 1.0) {
                return Err(ParsedLineError {
                    id: Some(id),
                    reason: RejectReason::StrengthOutOfRange,
                    detail: format!("line {line_number} cue strength must be in (0,1]"),
                    structural: false,
                });
            }
            Ok(EgArgs::Cue { concept, strength })
        }
        EgVerb::Link => {
            ensure_allowed_fields(object, &["a", "b", "weight"], line_number, id)?;
            let a = parse_required_string(object, "a", line_number)?;
            let b = parse_required_string(object, "b", line_number)?;
            let weight = parse_required_f32(object, "weight", line_number, id)?;
            if !(-1.0..=1.0).contains(&weight) {
                return Err(ParsedLineError {
                    id: Some(id),
                    reason: RejectReason::WeightOutOfRange,
                    detail: format!("line {line_number} link weight must be in [-1,1]"),
                    structural: false,
                });
            }
            Ok(EgArgs::Link { a, b, weight })
        }
        EgVerb::GoalPush | EgVerb::GoalPop => {
            ensure_allowed_fields(object, &["concept"], line_number, id)?;
            let concept = parse_required_string(object, "concept", line_number)?;
            Ok(EgArgs::Goal { concept })
        }
    }
}

fn commit_matching_staged(
    state: &mut AmState,
    staging: &mut StagingSidecar,
    verifier: &EgEnvelope,
    traces: &mut Vec<StepTrace>,
    verdicts: &mut Vec<EventVerdict>,
) -> Result<()> {
    let mut matching = Vec::new();
    let verifier_keys = corroboration_keys(verifier);
    if verifier_keys.is_empty() {
        return Ok(());
    }
    for (idx, entry) in staging.entries.iter().enumerate() {
        let staged_keys = corroboration_keys(&entry.event);
        if staged_keys.iter().any(|key| verifier_keys.contains(key)) {
            matching.push(idx);
        }
    }
    if matching.is_empty() {
        return Ok(());
    }

    let entries = matching
        .iter()
        .map(|idx| staging.entries[*idx].clone())
        .collect::<Vec<_>>();
    for idx in matching.into_iter().rev() {
        staging.entries.remove(idx);
    }
    for entry in entries {
        let trace = step_result(state, &entry.event.to_core_event()?)?;
        traces.push(trace);
        verdicts.push(EventVerdict {
            id: Some(entry.event.id),
            action: ApplyAction::CommittedFromStaging,
            source: Some(entry.event.source),
            reason: None,
            detail: None,
            committed_by: Some(verifier.id),
        });
    }
    staging.normalize();
    Ok(())
}

fn staged_entries_for(event: &EgEnvelope, staged_at_event_count: u64) -> Vec<StagedEntry> {
    match &event.args {
        EgArgs::Assert { concept, axes } => axes
            .iter()
            .map(|axis| StagedEntry {
                event: EgEnvelope {
                    id: event.id,
                    session_id: event.session_id.clone(),
                    source: event.source,
                    verb: event.verb,
                    args: EgArgs::Assert {
                        concept: concept.clone(),
                        axes: vec![axis.clone()],
                    },
                },
                staged_at_event_count,
            })
            .collect(),
        EgArgs::Link { .. } => vec![StagedEntry {
            event: event.clone(),
            staged_at_event_count,
        }],
        EgArgs::Cue { .. } | EgArgs::Goal { .. } => Vec::new(),
    }
}

fn expire_staged(staging: &mut StagingSidecar, verdicts: &mut Vec<EventVerdict>) {
    let applied = staging.applied_event_count;
    let mut retained = Vec::new();
    for entry in staging.entries.drain(..) {
        if applied.saturating_sub(entry.staged_at_event_count) >= STAGING_EXPIRY_APPLIED_EVENTS {
            verdicts.push(EventVerdict {
                id: Some(entry.event.id),
                action: ApplyAction::Expired,
                source: Some(entry.event.source),
                reason: None,
                detail: Some(
                    "expired after 200 subsequently applied non-staged events".to_string(),
                ),
                committed_by: None,
            });
            staging.tombstones.push(StagingTombstone {
                expired_at_event_count: applied,
                reason: "expired_after_200_applied_events".to_string(),
                staged_at_event_count: entry.staged_at_event_count,
                event: entry.event,
            });
        } else {
            retained.push(entry);
        }
    }
    staging.entries = retained;
    staging.normalize();
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CorroborationKey {
    Assert {
        concept: String,
        axis: String,
        sign: i8,
    },
    Link {
        a: String,
        b: String,
        sign: i8,
    },
}

fn corroboration_keys(event: &EgEnvelope) -> BTreeSet<CorroborationKey> {
    if !event.source.is_non_staged() && event.source != EventSource::LlmClaim {
        return BTreeSet::new();
    }
    match &event.args {
        EgArgs::Assert { concept, axes } => axes
            .iter()
            .map(|axis| CorroborationKey::Assert {
                concept: concept.clone(),
                axis: axis.axis.clone(),
                sign: sign(axis.value),
            })
            .collect(),
        EgArgs::Link { a, b, weight } => BTreeSet::from([CorroborationKey::Link {
            a: a.clone(),
            b: b.clone(),
            sign: sign(*weight),
        }]),
        EgArgs::Cue { .. } | EgArgs::Goal { .. } => BTreeSet::new(),
    }
}

fn sign(value: f32) -> i8 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

fn parse_required_u64(
    object: &Map<String, Value>,
    field: &str,
    line_number: usize,
) -> std::result::Result<u64, ParsedLineError> {
    let value = object.get(field).ok_or_else(|| ParsedLineError {
        id: None,
        reason: RejectReason::MissingField,
        detail: format!("line {line_number} missing field {field}"),
        structural: true,
    })?;
    value.as_u64().ok_or_else(|| ParsedLineError {
        id: None,
        reason: RejectReason::BadArgs,
        detail: format!("line {line_number} field {field} must be a u64"),
        structural: true,
    })
}

fn parse_required_string(
    object: &Map<String, Value>,
    field: &str,
    line_number: usize,
) -> std::result::Result<String, ParsedLineError> {
    let value = object.get(field).ok_or_else(|| ParsedLineError {
        id: parse_id_lossy(object),
        reason: RejectReason::MissingField,
        detail: format!("line {line_number} missing field {field}"),
        structural: true,
    })?;
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| ParsedLineError {
            id: parse_id_lossy(object),
            reason: RejectReason::BadArgs,
            detail: format!("line {line_number} field {field} must be a string"),
            structural: true,
        })
}

fn parse_required_f32(
    object: &Map<String, Value>,
    field: &str,
    line_number: usize,
    id: u64,
) -> std::result::Result<f32, ParsedLineError> {
    let value = object.get(field).ok_or_else(|| ParsedLineError {
        id: Some(id),
        reason: RejectReason::MissingField,
        detail: format!("line {line_number} missing field {field}"),
        structural: true,
    })?;
    let Some(number) = value.as_f64() else {
        return Err(ParsedLineError {
            id: Some(id),
            reason: RejectReason::NonFiniteValue,
            detail: format!("line {line_number} field {field} must be a finite number"),
            structural: false,
        });
    };
    if !number.is_finite() || number < f32::MIN as f64 || number > f32::MAX as f64 {
        return Err(ParsedLineError {
            id: Some(id),
            reason: RejectReason::NonFiniteValue,
            detail: format!("line {line_number} field {field} must be finite f32"),
            structural: false,
        });
    }
    Ok(number as f32)
}

fn ensure_allowed_fields(
    object: &Map<String, Value>,
    allowed: &[&str],
    line_number: usize,
    id: u64,
) -> std::result::Result<(), ParsedLineError> {
    let allowed = BTreeSet::from_iter(allowed.iter().copied());
    if let Some(field) = first_unknown_field(object, &allowed) {
        return Err(ParsedLineError {
            id: Some(id),
            reason: RejectReason::UnknownField,
            detail: format!("line {line_number} unknown args field {field}"),
            structural: true,
        });
    }
    Ok(())
}

fn parse_id_lossy(object: &Map<String, Value>) -> Option<u64> {
    object.get("id").and_then(Value::as_u64)
}

fn first_unknown_field<'a>(
    object: &'a Map<String, Value>,
    allowed: &BTreeSet<&str>,
) -> Option<&'a str> {
    object
        .keys()
        .map(String::as_str)
        .find(|field| !allowed.contains(field))
}

fn staged_key(entry: &StagedEntry) -> (u64, String, EgVerb, String) {
    (
        entry.event.id,
        entry.event.session_id.clone(),
        entry.event.verb,
        args_sort_key(&entry.event.args),
    )
}

fn staged_key_from_parts(event: &EgEnvelope) -> (u64, String, EgVerb, String) {
    (
        event.id,
        event.session_id.clone(),
        event.verb,
        args_sort_key(&event.args),
    )
}

fn args_sort_key(args: &EgArgs) -> String {
    match args {
        EgArgs::Assert { concept, axes } => {
            let mut parts = axes
                .iter()
                .map(|axis| format!("{}={}", axis.axis, axis.value))
                .collect::<Vec<_>>();
            parts.sort();
            format!("assert:{concept}:{}", parts.join(","))
        }
        EgArgs::Cue { concept, strength } => format!("cue:{concept}:{strength}"),
        EgArgs::Link { a, b, weight } => format!("link:{a}:{b}:{weight}"),
        EgArgs::Goal { concept } => format!("goal:{concept}"),
    }
}

fn sibling_with_added_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|file| file.to_os_string())
        .unwrap_or_default();
    name.push(suffix);
    path.with_file_name(name)
}
