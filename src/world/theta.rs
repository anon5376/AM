use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldTheta {
    pub width: i32,
    pub height: i32,
    pub portable_class_count: u32,
    pub barrier_class_count: u32,
    pub wall_count: usize,
    pub barriers_per_class: usize,
    pub portables_per_class: usize,
    pub consumable_count: usize,
    pub hazard_count: usize,
    pub exit_count: usize,
    pub start_energy_bucket: i32,
    pub min_energy_bucket: i32,
    pub max_energy_bucket: i32,
    pub consumable_energy: i32,
    pub hazard_energy: i32,
    pub step_cost_interval: u64,
    pub step_limit: usize,
    pub twins: bool,
    pub motion: bool,
    pub confound: bool,
    pub vision_radius: Option<i32>,
    pub rule_resample: bool,
}

impl Default for WorldTheta {
    fn default() -> Self {
        Self {
            width: 9,
            height: 7,
            portable_class_count: 2,
            barrier_class_count: 2,
            wall_count: 8,
            barriers_per_class: 1,
            portables_per_class: 1,
            consumable_count: 3,
            hazard_count: 2,
            exit_count: 1,
            start_energy_bucket: 10,
            min_energy_bucket: 0,
            max_energy_bucket: 20,
            consumable_energy: 3,
            hazard_energy: 2,
            step_cost_interval: 20,
            step_limit: 400,
            twins: false,
            motion: false,
            confound: false,
            vision_radius: None,
            rule_resample: false,
        }
    }
}

impl WorldTheta {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let text = fs::read_to_string(path.as_ref())
            .with_context(|| format!("read world theta {}", path.as_ref().display()))?;
        let theta: Self = serde_json::from_str(&text).context("parse world theta json")?;
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
        if self.twins {
            anyhow::bail!("UnimplementedTier: twins is not implemented in W0");
        }
        if self.motion {
            anyhow::bail!("UnimplementedTier: motion is not implemented in W0");
        }
        if self.confound {
            anyhow::bail!("UnimplementedTier: confound is not implemented in W0");
        }
        if self.vision_radius.is_some() {
            anyhow::bail!("UnimplementedTier: vision_radius is not implemented in W0");
        }
        if self.rule_resample {
            anyhow::bail!("UnimplementedTier: rule_resample is not implemented in W0");
        }

        anyhow::ensure!(self.width > 1, "world width must be greater than 1");
        anyhow::ensure!(self.height > 1, "world height must be greater than 1");
        anyhow::ensure!(
            self.portable_class_count > 0,
            "portable_class_count must be positive"
        );
        anyhow::ensure!(
            self.barrier_class_count > 0,
            "barrier_class_count must be positive"
        );
        anyhow::ensure!(
            self.portable_class_count == self.barrier_class_count,
            "portable and barrier class counts must match for the W0 permutation table"
        );
        anyhow::ensure!(self.exit_count <= 1, "W0 supports at most one exit");
        let cells = (self.width * self.height) as usize;
        anyhow::ensure!(
            self.entity_count() < cells,
            "world entity mix must fit with one empty self cell"
        );
        anyhow::ensure!(
            self.min_energy_bucket <= self.start_energy_bucket
                && self.start_energy_bucket <= self.max_energy_bucket,
            "start energy must be inside min/max buckets"
        );
        anyhow::ensure!(
            self.consumable_energy >= 0,
            "consumable_energy must be nonnegative"
        );
        anyhow::ensure!(self.hazard_energy >= 0, "hazard_energy must be nonnegative");
        anyhow::ensure!(
            self.step_cost_interval > 0,
            "step_cost_interval must be positive"
        );
        anyhow::ensure!(self.step_limit > 0, "step_limit must be positive");
        Ok(())
    }

    pub fn entity_count(&self) -> usize {
        self.wall_count
            + self.barriers_per_class * self.barrier_class_count as usize
            + self.portables_per_class * self.portable_class_count as usize
            + self.consumable_count
            + self.hazard_count
            + self.exit_count
    }

    pub fn hash(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("world theta json serialization cannot fail");
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}
