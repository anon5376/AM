use anyhow::Result;

pub fn chat(_prompt: &str) -> Result<String> {
    if std::env::var("AM_ENABLE_OLLAMA").ok().as_deref() != Some("1") {
        anyhow::bail!("AM_ENABLE_OLLAMA is not set to 1")
    }
    anyhow::bail!("Ollama client is a v0 seam stub")
}
