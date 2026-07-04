# AM 0.0.1

AM 0.0.1 is not a chatbot. The core never sees raw text, never calls an LLM, and never stores memories in a database.

Memory is the parameter state:

- `M`: concept coordinates
- `W`: sparse Hebbian links
- `a`: activation field
- `b`: consolidation baseline
- `V`: uncertainty variance
- `u`: last external touch tick

Text becomes validated Events outside the core. Persistence is array serialization. The trace is audit-only and is never read by the dynamics.

Basic commands:

```bash
cargo run -- init --snapshot data/am001.bin
cargo run -- step-text --snapshot data/am001.bin "assert rust truth_assert=1 goal_relevance=0.8"
cargo run -- step-text --snapshot data/am001.bin "cue rust 0.8" --diff
cargo run -- dump --snapshot data/am001.bin --sort act --top 20
```

Development dashboard:

```bash
cargo run -- dashboard --port 8765
```

Open `http://127.0.0.1:8765/`.

Model testing is on `http://127.0.0.1:8765/test.html`. It uses Ollama Cloud through the native Ollama API via the dashboard's same-origin proxy: `GET /api/tags` for the model dropdown and `POST /api/chat` for the event-shape test. Put the Ollama API key in the dashboard `Ollama API key` field.
