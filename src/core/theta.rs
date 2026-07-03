use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Theta {
    pub d: usize,
    pub n: usize,
    pub k: usize,
    pub a_max: usize,
    pub lam_a: f32,
    pub lam_b: f32,
    pub lam_settle: f32,
    pub lam_v: f32,
    pub eta_m: f32,
    pub eta_w: f32,
    pub del_w: f32,
    pub beta: f32,
    pub gamma: f32,
    pub k_i: f32,
    pub t: usize,
    pub eps: f32,
    pub th_act: f32,
    pub th_write: f32,
    pub th0: f32,
    pub tau_g: f32,
    pub a_init: f32,
    pub rho_b: f32,
    pub rho_c: f32,
    pub th_v: f32,
    pub th_v_close: f32,
    pub m_sign_window: usize,
    pub a_old: i64,
    pub th_gc: f32,
    pub th_merge: f32,
    pub w_max: f32,
    pub sigma: f32,
    pub v0: f32,
    pub b0: f32,
    pub eps_w: f32,
    pub eps_log: f32,
}

impl Default for Theta {
    fn default() -> Self {
        Self {
            d: 48,
            n: 4096,
            k: 32,
            a_max: 64,
            lam_a: 0.70,
            lam_b: 0.999,
            lam_settle: 0.55,
            lam_v: 0.8,
            eta_m: 0.15,
            eta_w: 0.30,
            del_w: 0.0,
            beta: 1.4,
            gamma: 1.2,
            k_i: 3.0,
            t: 20,
            eps: 1e-3,
            th_act: 0.25,
            th_write: 0.40,
            th0: 3.0,
            tau_g: 1.0,
            a_init: 0.6,
            rho_b: 0.01,
            rho_c: 0.15,
            th_v: 0.35,
            th_v_close: 0.20,
            m_sign_window: 5,
            a_old: 5000,
            th_gc: 0.05,
            th_merge: 0.92,
            w_max: 1.0,
            sigma: 0.0,
            v0: 0.20,
            b0: 0.05,
            eps_w: 1e-6,
            eps_log: 1e-6,
        }
    }
}

impl Theta {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let text = fs::read_to_string(path.as_ref())
            .with_context(|| format!("read theta {}", path.as_ref().display()))?;
        let theta: Self = serde_json::from_str(&text).context("parse theta json")?;
        theta.validate()?;
        Ok(theta)
    }

    pub fn load_optional(path: Option<&Path>) -> Result<Self> {
        match path {
            Some(path) => Self::from_path(path),
            None => {
                let theta = Self::default();
                theta.validate()?;
                Ok(theta)
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(self.d == 48, "theta.d must be 48");
        anyhow::ensure!(self.n > 0, "theta.n must be positive");
        anyhow::ensure!(self.k > 0, "theta.k must be positive");
        anyhow::ensure!(self.a_max > 0, "theta.a_max must be positive");
        anyhow::ensure!(
            self.lam_settle >= 0.0,
            "theta.lam_settle must be nonnegative"
        );
        anyhow::ensure!(self.lam_settle < 1.0, "theta.lam_settle must be < 1");
        anyhow::ensure!(self.beta >= 0.0, "theta.beta must be nonnegative");
        anyhow::ensure!(self.k_i >= 0.0, "theta.k_i must be nonnegative");
        anyhow::ensure!(self.th_act > 0.0, "theta.th_act must be positive");
        anyhow::ensure!(self.sigma == 0.0, "theta.sigma must be 0 for v0");
        anyhow::ensure!(self.tau_g > 0.0, "theta.tau_g must be positive");
        anyhow::ensure!(self.eps_log >= 0.0, "theta.eps_log must be nonnegative");
        let resting = self.resting_activation_bound();
        anyhow::ensure!(
            resting < self.th_act,
            "theta violates A15c resting-field invariant: beta*sigmoid(-th0)/(1-lam_settle) = {} >= th_act {}",
            resting,
            self.th_act
        );
        Ok(())
    }

    pub fn resting_activation_bound(&self) -> f32 {
        let sigmoid = 1.0 / (1.0 + self.th0.exp());
        self.beta * sigmoid / (1.0 - self.lam_settle)
    }

    pub fn hash(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("theta json serialization cannot fail");
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}
