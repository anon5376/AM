pub mod corpus;
pub mod prompt;
pub mod results;
pub mod scoring;
pub mod transport;

use crate::beval::corpus::{load_corpus, Category, Task};
use crate::beval::prompt::{build_prompt, context_token_count, load_b1_context, LaneContext};
use crate::beval::results::{
    BevalResults, CategoryScore, DriftScore, RunMetadata, TaskResult, TokenCost, TransportMetadata,
};
use crate::beval::scoring::score_response;
use crate::beval::transport::{
    prompt_hash, write_recorded_fixture, FixtureManifest, LlmTransport, ReplayTransport,
};
use crate::llm::ollama_client::OllamaTransport;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub const RESULTS_SCHEMA: &str = "BEVAL-1";
pub const B1_CONTEXT_BUDGET_TOKENS: usize = 8_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lane {
    B0,
    B1,
    B2,
}

impl Lane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::B0 => "b0",
            Self::B1 => "b1",
            Self::B2 => "b2",
        }
    }
}

impl fmt::Display for Lane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Lane {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "b0" => Ok(Self::B0),
            "b1" => Ok(Self::B1),
            "b2" => Ok(Self::B2),
            _ => anyhow::bail!("unknown beval lane `{value}`; expected b0, b1, or b2"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    Replay,
    Live,
}

impl TransportKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Replay => "replay",
            Self::Live => "live",
        }
    }
}

impl FromStr for TransportKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "replay" => Ok(Self::Replay),
            "live" => Ok(Self::Live),
            _ => anyhow::bail!("unknown beval transport `{value}`; expected replay or live"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BevalConfig {
    pub corpus_dir: PathBuf,
    pub lane: Lane,
    pub transport: TransportKind,
    pub fixtures_dir: PathBuf,
    pub record: bool,
    pub out: PathBuf,
}

pub fn run_beval(config: &BevalConfig) -> Result<BevalResults> {
    let tasks = load_corpus(&config.corpus_dir)?;
    let results = match config.lane {
        Lane::B2 => run_lane_unavailable(config, &tasks)?,
        Lane::B0 | Lane::B1 => match config.transport {
            TransportKind::Replay => {
                if config.record {
                    anyhow::bail!("--record is only valid with --transport live");
                }
                let mut transport = ReplayTransport::new(&config.fixtures_dir)?;
                run_lane(config, &tasks, &mut transport, None)?
            }
            TransportKind::Live => {
                let mut transport = OllamaTransport::from_env()?;
                run_lane(config, &tasks, &mut transport, Some(&config.fixtures_dir))?
            }
        },
    };
    write_results(&config.out, &results)?;
    Ok(results)
}

fn run_lane<T: LlmTransport>(
    config: &BevalConfig,
    tasks: &[Task],
    transport: &mut T,
    record_dir: Option<&Path>,
) -> Result<BevalResults> {
    let context = match config.lane {
        Lane::B0 => LaneContext::empty(),
        Lane::B1 => load_b1_context(&config.fixtures_dir, B1_CONTEXT_BUDGET_TOKENS)?,
        Lane::B2 => unreachable!("B2 is handled as LaneUnavailable in B02"),
    };

    let mut task_results = Vec::new();
    let mut total_context_tokens = 0_usize;
    for task in tasks {
        if task
            .requires_lane
            .is_some_and(|required| required != config.lane)
        {
            task_results.push(TaskResult::skipped(
                task,
                config.lane,
                "RequiresDifferentLane",
            ));
            continue;
        }

        let prompt = build_prompt(config.lane, task, &context);
        let hash = prompt_hash(&prompt);
        let raw_response = transport.complete(&prompt)?;
        if config.record {
            let dir = record_dir.context("--record requires a fixture directory")?;
            write_recorded_fixture(dir, &hash, &raw_response)?;
        }
        let score = score_response(task, &raw_response);
        total_context_tokens += context_token_count(&context);
        task_results.push(TaskResult::evaluated(
            task,
            config.lane,
            hash,
            context_token_count(&context),
            raw_response,
            score,
        ));
    }

    let metadata = RunMetadata {
        schema: RESULTS_SCHEMA.to_string(),
        lane: config.lane,
        transport: config.transport,
        transport_metadata: transport.metadata(),
        corpus_task_count: tasks.len(),
    };
    Ok(assemble_results(
        metadata,
        task_results,
        total_context_tokens,
    ))
}

fn run_lane_unavailable(config: &BevalConfig, tasks: &[Task]) -> Result<BevalResults> {
    let manifest = if config.transport == TransportKind::Replay {
        Some(FixtureManifest::load(&config.fixtures_dir)?)
    } else {
        None
    };
    let task_results = tasks
        .iter()
        .map(|task| TaskResult::skipped(task, Lane::B2, "LaneUnavailable"))
        .collect();
    let transport_metadata = if let Some(manifest) = manifest {
        TransportMetadata::replay(manifest.synthetic, manifest.version)
    } else {
        TransportMetadata::lane_unavailable()
    };
    let metadata = RunMetadata {
        schema: RESULTS_SCHEMA.to_string(),
        lane: Lane::B2,
        transport: config.transport,
        transport_metadata,
        corpus_task_count: tasks.len(),
    };
    Ok(assemble_results(metadata, task_results, 0))
}

fn assemble_results(
    metadata: RunMetadata,
    task_results: Vec<TaskResult>,
    total_context_tokens: usize,
) -> BevalResults {
    let mut category_totals: BTreeMap<Category, (usize, usize)> = BTreeMap::new();
    let mut stale_total = 0_usize;
    let mut stale_repeats = 0_usize;
    let mut evaluated = 0_usize;
    let mut skipped = 0_usize;

    for result in &task_results {
        if result.skipped {
            skipped += 1;
            continue;
        }
        evaluated += 1;
        let entry = category_totals
            .entry(result.category)
            .or_insert((0_usize, 0_usize));
        entry.1 += 1;
        if result.matched {
            entry.0 += 1;
        }
        if result.category == Category::StaleClaim {
            stale_total += 1;
            if result.stale_repeated {
                stale_repeats += 1;
            }
        }
    }

    let category_scores = Category::all()
        .iter()
        .map(|category| {
            let (matched, total) = category_totals
                .get(category)
                .copied()
                .unwrap_or((0_usize, 0_usize));
            CategoryScore {
                category: *category,
                matched,
                total,
                accuracy: ratio(matched, total),
            }
        })
        .collect();

    let drift = compute_drift(&task_results);
    let mean_context_tokens = ratio(total_context_tokens, evaluated);
    BevalResults {
        metadata,
        evaluated,
        skipped,
        category_scores,
        stale_claim_rate: ratio(stale_repeats, stale_total),
        stale_repeats,
        stale_total,
        drift,
        token_cost: TokenCost {
            mean_context_tokens,
            total_context_tokens,
        },
        task_results,
    }
}

fn compute_drift(task_results: &[TaskResult]) -> DriftScore {
    let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for result in task_results {
        if result.skipped || result.category != Category::Drift {
            continue;
        }
        if let Some(group) = &result.drift_group {
            if let Some(answer) = &result.answer {
                groups
                    .entry(group.clone())
                    .or_default()
                    .insert(answer.clone());
            }
        }
    }
    let evaluated_groups = groups.len();
    let mismatched_groups = groups.values().filter(|answers| answers.len() > 1).count();
    DriftScore {
        evaluated_groups,
        mismatched_groups,
        drift_rate: ratio(mismatched_groups, evaluated_groups),
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn write_results(path: &Path, results: &BevalResults) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(results).context("serialize beval results")?;
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::ratio;

    #[test]
    fn ratio_zero_denominator_is_zero() {
        assert_eq!(ratio(7, 0), 0.0);
    }
}
