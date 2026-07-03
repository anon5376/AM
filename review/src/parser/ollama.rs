use crate::core::event::Event;
use anyhow::Result;

pub fn parse_with_ollama(_text: &str, _event_id: i64) -> Result<Event> {
    anyhow::bail!("Ollama parser seam is stubbed for v0; use parser::rule")
}
