use crate::beval::results::TransportMetadata;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub trait LlmTransport {
    fn complete(&mut self, prompt: &str) -> Result<String>;
    fn metadata(&self) -> TransportMetadata;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FixtureManifest {
    pub version: String,
    pub synthetic: bool,
    pub fixture_format: String,
}

impl FixtureManifest {
    pub fn load(fixtures_dir: &Path) -> Result<Self> {
        let path = fixtures_dir.join("manifest.json");
        let text = fs::read_to_string(&path)
            .with_context(|| format!("read fixture manifest {}", path.display()))?;
        let manifest: Self = serde_json::from_str(&text)
            .with_context(|| format!("parse fixture manifest {}", path.display()))?;
        anyhow::ensure!(
            manifest.fixture_format == "sha256-prompt-v1",
            "unsupported fixture_format {}",
            manifest.fixture_format
        );
        Ok(manifest)
    }
}

#[derive(Clone, Debug)]
pub struct ReplayTransport {
    fixtures_dir: PathBuf,
    manifest: FixtureManifest,
}

impl ReplayTransport {
    pub fn new(fixtures_dir: &Path) -> Result<Self> {
        Ok(Self {
            fixtures_dir: fixtures_dir.to_path_buf(),
            manifest: FixtureManifest::load(fixtures_dir)?,
        })
    }
}

impl LlmTransport for ReplayTransport {
    fn complete(&mut self, prompt: &str) -> Result<String> {
        let hash = prompt_hash(prompt);
        let path = self.fixtures_dir.join(format!("{hash}.txt"));
        if !path.exists() {
            anyhow::bail!(
                "missing replay fixture hash={} prompt_preview={}",
                hash,
                prompt_preview(prompt)
            );
        }
        fs::read_to_string(&path).with_context(|| format!("read fixture {}", path.display()))
    }

    fn metadata(&self) -> TransportMetadata {
        TransportMetadata::replay(self.manifest.synthetic, self.manifest.version.clone())
    }
}

pub fn prompt_hash(prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn prompt_preview(prompt: &str) -> String {
    prompt
        .chars()
        .take(120)
        .collect::<String>()
        .replace('\n', "\\n")
}

pub fn write_recorded_fixture(fixtures_dir: &Path, hash: &str, response: &str) -> Result<()> {
    fs::create_dir_all(fixtures_dir)
        .with_context(|| format!("create fixtures dir {}", fixtures_dir.display()))?;
    let path = fixtures_dir.join(format!("{hash}.txt"));
    fs::write(&path, response).with_context(|| format!("write fixture {}", path.display()))?;
    let manifest = FixtureManifest {
        version: "synthetic_v1".to_string(),
        synthetic: false,
        fixture_format: "sha256-prompt-v1".to_string(),
    };
    let manifest_path = fixtures_dir.join("manifest.json");
    let bytes = serde_json::to_vec_pretty(&manifest).context("serialize fixture manifest")?;
    fs::write(&manifest_path, bytes)
        .with_context(|| format!("write fixture manifest {}", manifest_path.display()))
}

#[cfg(test)]
mod tests {
    use super::{prompt_hash, prompt_preview};

    #[test]
    fn hash_is_stable() {
        assert_eq!(
            prompt_hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn preview_is_bounded_and_single_line() {
        let preview = prompt_preview(&format!("{}\n{}", "a".repeat(130), "tail"));
        assert_eq!(preview.len(), 120);
        assert!(!preview.contains('\n'));
    }
}
