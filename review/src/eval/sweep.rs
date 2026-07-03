use crate::core::theta::Theta;
use crate::eval::criteria::{run_core_criteria_with_config, CriteriaConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweepGrid {
    pub lam_settle: Vec<f32>,
    pub beta: Vec<f32>,
    pub gamma: Vec<f32>,
    pub th0: Vec<f32>,
    #[serde(default = "default_k_i_values")]
    pub k_i: Vec<f32>,
    pub eta_w: Vec<f32>,
    pub del_w: Vec<f32>,
    #[serde(default)]
    pub th_write: Option<Vec<f32>>,
    #[serde(default)]
    pub rho_b: Option<Vec<f32>>,
    #[serde(default)]
    pub a_init: Option<Vec<f32>>,
    #[serde(default)]
    pub b0: Option<Vec<f32>>,
    #[serde(default)]
    pub th_act: Option<Vec<f32>>,
    #[serde(default = "default_t")]
    pub t: usize,
    #[serde(default)]
    pub scale: SweepScale,
}

fn default_t() -> usize {
    20
}

fn default_k_i_values() -> Vec<f32> {
    vec![Theta::default().k_i]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweepScale {
    pub n: usize,
    pub completion_assemblies: usize,
}

impl Default for SweepScale {
    fn default() -> Self {
        Self {
            n: 512,
            completion_assemblies: 12,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SweepSummary {
    pub total_points: usize,
    pub invalid_points: usize,
    pub evaluated_points: usize,
    pub all_pass_points: usize,
}

impl Default for SweepGrid {
    fn default() -> Self {
        Self {
            lam_settle: vec![0.2, 0.35, 0.55],
            beta: vec![1.0, 1.4, 1.8],
            gamma: vec![0.6, 0.9, 1.2],
            th0: vec![1.0, 2.0, 3.0, 4.0],
            k_i: vec![1.0, 2.0, 3.0],
            eta_w: vec![0.05, 0.15, 0.3],
            del_w: vec![0.0, 0.002],
            th_write: None,
            rho_b: None,
            a_init: None,
            b0: None,
            th_act: None,
            t: 20,
            scale: SweepScale::default(),
        }
    }
}

pub fn load_grid(path: &Path) -> Result<SweepGrid> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).context("parse sweep grid")
}

pub fn run_sweep(grid: &SweepGrid, out: &Path) -> Result<SweepSummary> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut summary = SweepSummary::default();
    let mut csv = String::new();
    csv.push_str("theta_hash,n,completion_assemblies,valid,lam_settle,beta,gamma,th0,k_i,eta_w,del_w,th_write,rho_b,a_init,b0,th_act,t,determinism,completion,reinforcement,contradiction,forgetting,diff_integrity,all_pass,completion_recall,completion_contamination,failures\n");

    let base = Theta::default();
    let th_write_values = values_or_default(&grid.th_write, base.th_write);
    let rho_b_values = values_or_default(&grid.rho_b, base.rho_b);
    let a_init_values = values_or_default(&grid.a_init, base.a_init);
    let b0_values = values_or_default(&grid.b0, base.b0);
    let th_act_values = values_or_default(&grid.th_act, base.th_act);

    for lam_settle in &grid.lam_settle {
        for beta in &grid.beta {
            for gamma in &grid.gamma {
                for th0 in &grid.th0 {
                    for k_i in &grid.k_i {
                        for eta_w in &grid.eta_w {
                            for del_w in &grid.del_w {
                                for th_write in &th_write_values {
                                    for rho_b in &rho_b_values {
                                        for a_init in &a_init_values {
                                            for b0 in &b0_values {
                                                for th_act in &th_act_values {
                                                    append_sweep_row(
                                                        grid,
                                                        &mut summary,
                                                        &mut csv,
                                                        *lam_settle,
                                                        *beta,
                                                        *gamma,
                                                        *th0,
                                                        *k_i,
                                                        *eta_w,
                                                        *del_w,
                                                        *th_write,
                                                        *rho_b,
                                                        *a_init,
                                                        *b0,
                                                        *th_act,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    fs::write(out, csv).with_context(|| format!("write {}", out.display()))?;
    Ok(summary)
}

#[allow(clippy::too_many_arguments)]
fn append_sweep_row(
    grid: &SweepGrid,
    summary: &mut SweepSummary,
    csv: &mut String,
    lam_settle: f32,
    beta: f32,
    gamma: f32,
    th0: f32,
    k_i: f32,
    eta_w: f32,
    del_w: f32,
    th_write: f32,
    rho_b: f32,
    a_init: f32,
    b0: f32,
    th_act: f32,
) {
    summary.total_points += 1;
    let theta = Theta {
        n: grid.scale.n,
        lam_settle,
        beta,
        gamma,
        th0,
        k_i,
        eta_w,
        del_w,
        th_write,
        rho_b,
        a_init,
        b0,
        th_act,
        t: grid.t,
        ..Theta::default()
    };
    let mut pass = std::collections::BTreeMap::new();
    let mut failures = Vec::new();
    let mut recall = 0.0;
    let mut contamination = 0.0;
    let mut valid = true;
    let mut all_pass = false;

    match theta.validate() {
        Ok(()) => {
            summary.evaluated_points += 1;
            let results = run_core_criteria_with_config(
                &theta,
                CriteriaConfig {
                    completion_assemblies: grid.scale.completion_assemblies,
                },
            );
            for result in &results {
                pass.insert(result.name, result.passed);
                if !result.passed {
                    failures.push(format!("{}:{}", result.name, result.detail));
                }
                if result.name == "completion" {
                    recall = *result.metrics.get("recall").unwrap_or(&0.0);
                    contamination = *result.metrics.get("contamination").unwrap_or(&0.0);
                }
            }
            all_pass = results.iter().all(|result| result.passed);
            if all_pass {
                summary.all_pass_points += 1;
            }
        }
        Err(err) => {
            valid = false;
            summary.invalid_points += 1;
            failures.push(format!("invalid:{err}"));
        }
    }

    csv.push_str(&format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},\"{}\"\n",
        theta.hash(),
        theta.n,
        grid.scale.completion_assemblies,
        valid,
        lam_settle,
        beta,
        gamma,
        th0,
        k_i,
        eta_w,
        del_w,
        th_write,
        rho_b,
        a_init,
        b0,
        th_act,
        grid.t,
        pass.get("determinism").copied().unwrap_or(false),
        pass.get("completion").copied().unwrap_or(false),
        pass.get("reinforcement").copied().unwrap_or(false),
        pass.get("contradiction").copied().unwrap_or(false),
        pass.get("forgetting").copied().unwrap_or(false),
        pass.get("diff_integrity").copied().unwrap_or(false),
        all_pass,
        recall,
        contamination,
        failures.join(";").replace('"', "'")
    ));
}

fn values_or_default(values: &Option<Vec<f32>>, default: f32) -> Vec<f32> {
    values.clone().unwrap_or_else(|| vec![default])
}
