use crate::beval::Lane;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    StageAccuracy,
    StaleClaim,
    ContradictionHandling,
    TestGrounded,
    Drift,
}

impl Category {
    pub fn all() -> &'static [Self] {
        &[
            Self::StageAccuracy,
            Self::StaleClaim,
            Self::ContradictionHandling,
            Self::TestGrounded,
            Self::Drift,
        ]
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StageAccuracy => "stage_accuracy",
            Self::StaleClaim => "stale_claim",
            Self::ContradictionHandling => "contradiction_handling",
            Self::TestGrounded => "test_grounded",
            Self::Drift => "drift",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub category: Category,
    pub question: String,
    pub answer_key: AnswerKey,
    pub source_ref: String,
    #[serde(default)]
    pub requires_lane: Option<Lane>,
    #[serde(default)]
    pub drift_group: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnswerKey {
    pub matchers: Vec<Matcher>,
    #[serde(default)]
    pub stale_markers: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Matcher {
    Exact { value: String },
    Regex { pattern: String },
    MustContainAll { values: Vec<String> },
    MustContainNone { values: Vec<String> },
}

pub fn load_corpus(dir: &Path) -> Result<Vec<Task>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read corpus dir {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();

    let mut tasks = Vec::new();
    for path in paths {
        let text =
            fs::read_to_string(&path).with_context(|| format!("read task {}", path.display()))?;
        let mut task: Task =
            serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        normalize_task_path_fields(&mut task, &path);
        tasks.push(task);
    }
    validate_corpus(&tasks)?;
    Ok(tasks)
}

pub fn validate_corpus(tasks: &[Task]) -> Result<()> {
    anyhow::ensure!(
        (25..=40).contains(&tasks.len()),
        "B02 corpus must contain 25-40 tasks, found {}",
        tasks.len()
    );

    let mut ids = BTreeSet::new();
    let mut counts: BTreeMap<Category, usize> = BTreeMap::new();
    let mut drift_groups: BTreeSet<String> = BTreeSet::new();
    for task in tasks {
        anyhow::ensure!(!task.id.trim().is_empty(), "task id is empty");
        anyhow::ensure!(ids.insert(task.id.clone()), "duplicate task id {}", task.id);
        anyhow::ensure!(
            !task.question.trim().is_empty(),
            "task {} has empty question",
            task.id
        );
        anyhow::ensure!(
            !task.source_ref.trim().is_empty(),
            "task {} has empty source_ref",
            task.id
        );
        validate_answer_key(&task.id, &task.answer_key)?;
        *counts.entry(task.category).or_insert(0) += 1;
        if task.category == Category::Drift {
            let group = task
                .drift_group
                .as_ref()
                .with_context(|| format!("drift task {} missing drift_group", task.id))?;
            drift_groups.insert(group.clone());
        }
        if task.category == Category::ContradictionHandling {
            anyhow::ensure!(
                task.requires_lane == Some(Lane::B2),
                "contradiction task {} must be requires_lane=b2 in B02",
                task.id
            );
        }
    }

    require_count(&counts, Category::StageAccuracy, 8)?;
    require_count(&counts, Category::StaleClaim, 5)?;
    require_count(&counts, Category::ContradictionHandling, 5)?;
    require_count(&counts, Category::TestGrounded, 5)?;
    anyhow::ensure!(
        drift_groups.len() >= 3,
        "drift category must contain at least 3 repeated-question groups, found {}",
        drift_groups.len()
    );
    Ok(())
}

fn validate_answer_key(task_id: &str, key: &AnswerKey) -> Result<()> {
    anyhow::ensure!(
        !key.matchers.is_empty(),
        "task {task_id} has no deterministic matchers"
    );
    for matcher in &key.matchers {
        match matcher {
            Matcher::Exact { value } => {
                anyhow::ensure!(!value.is_empty(), "task {task_id} exact matcher is empty");
            }
            Matcher::Regex { pattern } => {
                anyhow::ensure!(!pattern.is_empty(), "task {task_id} regex matcher is empty");
            }
            Matcher::MustContainAll { values } | Matcher::MustContainNone { values } => {
                anyhow::ensure!(
                    !values.is_empty(),
                    "task {task_id} contains matcher has no values"
                );
                anyhow::ensure!(
                    values.iter().all(|value| !value.is_empty()),
                    "task {task_id} contains matcher has an empty value"
                );
            }
        }
    }
    Ok(())
}

fn require_count(
    counts: &BTreeMap<Category, usize>,
    category: Category,
    minimum: usize,
) -> Result<()> {
    let found = counts.get(&category).copied().unwrap_or(0);
    anyhow::ensure!(
        found >= minimum,
        "category {category} requires at least {minimum} tasks, found {found}"
    );
    Ok(())
}

fn normalize_task_path_fields(task: &mut Task, path: &Path) {
    if task.id.trim().is_empty() {
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            task.id = stem.to_string();
        }
    }
}
