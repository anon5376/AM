use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldTheta {
    pub width: i32,
    pub height: i32,
    pub entity_count: usize,
    pub start_energy_bucket: i32,
    pub min_energy_bucket: i32,
    pub max_energy_bucket: i32,
    pub step_limit: usize,
    pub touch_reward: i32,
    pub block_reward: i32,
}

impl Default for WorldTheta {
    fn default() -> Self {
        Self {
            width: 7,
            height: 5,
            entity_count: 8,
            start_energy_bucket: 5,
            min_energy_bucket: 0,
            max_energy_bucket: 9,
            step_limit: 256,
            touch_reward: 1,
            block_reward: -1,
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
        anyhow::ensure!(self.width > 1, "world width must be greater than 1");
        anyhow::ensure!(self.height > 1, "world height must be greater than 1");
        let cells = (self.width * self.height) as usize;
        anyhow::ensure!(
            self.entity_count < cells,
            "entity_count must fit with one empty self cell"
        );
        anyhow::ensure!(
            self.min_energy_bucket <= self.start_energy_bucket
                && self.start_energy_bucket <= self.max_energy_bucket,
            "start energy must be inside min/max buckets"
        );
        anyhow::ensure!(self.step_limit > 0, "step_limit must be positive");
        Ok(())
    }

    pub fn hash(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("world theta json serialization cannot fail");
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}
