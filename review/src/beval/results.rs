use crate::beval::corpus::{Category, Task};
use crate::beval::scoring::ScoreOutcome;
use crate::beval::{Lane, TransportKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BevalResults {
    pub metadata: RunMetadata,
    pub evaluated: usize,
    pub skipped: usize,
    pub category_scores: Vec<CategoryScore>,
    pub stale_claim_rate: f64,
    pub stale_repeats: usize,
    pub stale_total: usize,
    pub drift: DriftScore,
    pub token_cost: TokenCost,
    pub task_results: Vec<TaskResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunMetadata {
    pub schema: String,
    pub lane: Lane,
    pub transport: TransportKind,
    pub transport_metadata: TransportMetadata,
    pub corpus_task_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransportMetadata {
    pub kind: String,
    pub synthetic: Option<bool>,
    pub fixture_manifest_version: Option<String>,
    pub model: Option<String>,
    pub digest: Option<String>,
    pub temperature: Option<f32>,
    pub seed: Option<u64>,
    pub endpoint: Option<String>,
}

impl TransportMetadata {
    pub fn replay(synthetic: bool, fixture_manifest_version: String) -> Self {
        Self {
            kind: "replay".to_string(),
            synthetic: Some(synthetic),
            fixture_manifest_version: Some(fixture_manifest_version),
            model: None,
            digest: None,
            temperature: None,
            seed: None,
            endpoint: None,
        }
    }

    pub fn live(
        model: String,
        digest: String,
        temperature: f32,
        seed: Option<u64>,
        endpoint: String,
    ) -> Self {
        Self {
            kind: "live".to_string(),
            synthetic: Some(false),
            fixture_manifest_version: None,
            model: Some(model),
            digest: Some(digest),
            temperature: Some(temperature),
            seed,
            endpoint: Some(endpoint),
        }
    }

    pub fn lane_unavailable() -> Self {
        Self {
            kind: "lane_unavailable".to_string(),
            synthetic: None,
            fixture_manifest_version: None,
            model: None,
            digest: None,
            temperature: None,
            seed: None,
            endpoint: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryScore {
    pub category: Category,
    pub matched: usize,
    pub total: usize,
    pub accuracy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DriftScore {
    pub evaluated_groups: usize,
    pub mismatched_groups: usize,
    pub drift_rate: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenCost {
    pub mean_context_tokens: f64,
    pub total_context_tokens: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResult {
    pub id: String,
    pub category: Category,
    pub lane: Lane,
    pub skipped: bool,
    pub skip_reason: Option<String>,
    pub prompt_hash: Option<String>,
    pub context_tokens: usize,
    pub matched: bool,
    pub answer: Option<String>,
    pub reasons: Vec<String>,
    pub stale_repeated: bool,
    pub drift_group: Option<String>,
}

impl TaskResult {
    pub fn skipped(task: &Task, lane: Lane, reason: &str) -> Self {
        Self {
            id: task.id.clone(),
            category: task.category,
            lane,
            skipped: true,
            skip_reason: Some(reason.to_string()),
            prompt_hash: None,
            context_tokens: 0,
            matched: false,
            answer: None,
            reasons: Vec::new(),
            stale_repeated: false,
            drift_group: task.drift_group.clone(),
        }
    }

    pub fn evaluated(
        task: &Task,
        lane: Lane,
        prompt_hash: String,
        context_tokens: usize,
        _raw_response: String,
        score: ScoreOutcome,
    ) -> Self {
        Self {
            id: task.id.clone(),
            category: task.category,
            lane,
            skipped: false,
            skip_reason: None,
            prompt_hash: Some(prompt_hash),
            context_tokens,
            matched: score.matched,
            answer: score.answer,
            reasons: score.reasons,
            stale_repeated: score.stale_repeated,
            drift_group: task.drift_group.clone(),
        }
    }
}
