use crate::core::theta::Theta;
use crate::eval::criteria::run_core_criteria;
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
}

fn default_t() -> usize {
    20
}

impl Default for SweepGrid {
    fn default() -> Self {
        Self {
            lam_settle: vec![0.2, 0.35, 0.55],
            beta: vec![1.0, 1.4, 1.8],
            gamma: vec![0.6, 0.9, 1.2],
            th0: vec![0.5, 1.0, 2.0],
            eta_w: vec![0.05, 0.15, 0.3],
            del_w: vec![0.0, 0.001, 0.002],
            th_write: None,
            rho_b: None,
            a_init: None,
            b0: None,
            th_act: None,
            t: 20,
        }
    }
}

pub fn load_grid(path: &Path) -> Result<SweepGrid> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).context("parse sweep grid")
}

pub fn run_sweep(grid: &SweepGrid, out: &Path) -> Result<()> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut csv = String::new();
    csv.push_str("theta_hash,lam_settle,beta,gamma,th0,eta_w,del_w,th_write,rho_b,a_init,b0,th_act,t,determinism,completion,reinforcement,contradiction,forgetting,diff_integrity,all_pass,completion_recall,completion_contamination,failures\n");

    for lam_settle in &grid.lam_settle {
        for beta in &grid.beta {
            for gamma in &grid.gamma {
                for th0 in &grid.th0 {
                    for eta_w in &grid.eta_w {
                        for del_w in &grid.del_w {
                            let base = Theta::default();
                            let th_write_values = values_or_default(&grid.th_write, base.th_write);
                            let rho_b_values = values_or_default(&grid.rho_b, base.rho_b);
                            let a_init_values = values_or_default(&grid.a_init, base.a_init);
                            let b0_values = values_or_default(&grid.b0, base.b0);
                            let th_act_values = values_or_default(&grid.th_act, base.th_act);
                            for th_write in &th_write_values {
                                for rho_b in &rho_b_values {
                                    for a_init in &a_init_values {
                                        for b0 in &b0_values {
                                            for th_act in &th_act_values {
                                                let theta = Theta {
                                                    lam_settle: *lam_settle,
                                                    beta: *beta,
                                                    gamma: *gamma,
                                                    th0: *th0,
                                                    eta_w: *eta_w,
                                                    del_w: *del_w,
                                                    th_write: *th_write,
                                                    rho_b: *rho_b,
                                                    a_init: *a_init,
                                                    b0: *b0,
                                                    th_act: *th_act,
                                                    t: grid.t,
                                                    ..Theta::default()
                                                };
                                                let results = run_core_criteria(&theta);
                                                let mut pass = std::collections::BTreeMap::new();
                                                let mut failures = Vec::new();
                                                let mut recall = 0.0;
                                                let mut contamination = 0.0;
                                                for result in &results {
                                                    pass.insert(result.name, result.passed);
                                                    if !result.passed {
                                                        failures.push(format!(
                                                            "{}:{}",
                                                            result.name, result.detail
                                                        ));
                                                    }
                                                    if result.name == "completion" {
                                                        recall = *result
                                                            .metrics
                                                            .get("recall")
                                                            .unwrap_or(&0.0);
                                                        contamination = *result
                                                            .metrics
                                                            .get("contamination")
                                                            .unwrap_or(&0.0);
                                                    }
                                                }
                                                let all_pass =
                                                    results.iter().all(|result| result.passed);
                                                csv.push_str(&format!(
                                                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},\"{}\"\n",
                                                    theta.hash(),
                                                    lam_settle,
                                                    beta,
                                                    gamma,
                                                    th0,
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
    fs::write(out, csv).with_context(|| format!("write {}", out.display()))
}

fn values_or_default(values: &Option<Vec<f32>>, default: f32) -> Vec<f32> {
    values.clone().unwrap_or_else(|| vec![default])
}
