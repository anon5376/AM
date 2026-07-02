# Ollama Seam

AM001 v0 keeps Ollama outside the core.

- `src/llm/ollama_client.rs` is a gated stub.
- `src/parser/ollama.rs` is a gated parser seam stub.
- The core never calls either module.
- Daemon ticks never call an LLM.
- Text must become a validated `Event` before `step()` receives it.

Future implementation contract:

- Require `AM_ENABLE_OLLAMA=1`.
- Read `OLLAMA_HOST` and `AM_OLLAMA_MODEL`.
- POST to `/api/chat` with `stream:false`.
- Parse model output as Event JSON.
- Run the same validator used by rule/parser JSON events.

