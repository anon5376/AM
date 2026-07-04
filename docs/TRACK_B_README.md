# Track B Local Flow

Track B is a deterministic bridge around AM001. It can compile bounded context, stage proposed LLM claims, ingest verified artifacts, inspect provenance, and render a static dashboard. It does not call an LLM unless a future user-present live eval explicitly enables the dormant Ollama seam.

## Core Commands

Compile a bounded snapshot context:

```text
am compile-context --snapshot am.bin --budget 1200 --out context.txt
```

Run B-eval replay with compiled B2 context:

```text
am beval --corpus beval/corpus --lane b2 --transport replay --fixtures beval/fixtures/synthetic_v1 --snapshot am.bin --out b2.json
```

Distill one fenced `eg1` JSONL block into staged `llm_claim` events:

```text
am distill --input llm_output.txt --session session_1 --events-out claims.jsonl
```

Ingest verified artifacts into `test_verified` events:

```text
am ingest --kind cargo-test --input cargo-test.txt --session session_1 --events-out verified.jsonl
am ingest --kind sweep-csv --input results.csv --session session_1 --events-out verified.jsonl
am ingest --kind world-run --input world-run.txt --session session_1 --events-out verified.jsonl
```

Apply normally or dry-run:

```text
am apply --snapshot am.bin --events claims.jsonl --report apply.json
am apply --dry-run --snapshot am.bin --events claims.jsonl --report preview.json
```

Inspect trace provenance:

```text
am provenance --snapshot am.bin --event 1
am provenance --trace am.bin.trace.jsonl --event 1
```

Render a static dashboard:

```text
am dashboard --snapshot am.bin --beval b2.json --trace am.bin.trace.jsonl --report apply.json --out dashboard.html
```

## Boundaries

- `llm_claim` events never mutate AM directly. They stage, then commit only after `user` or `test_verified` corroboration through EG-1.
- Distill rejects missing, duplicate, malformed, or non-validating `eg1` blocks before writing any output.
- Ingest stores compact verification summaries, not raw test/world/sweep text.
- Dashboard output is read-only, self-contained HTML. It is inspection, not evidence by itself.
- No database, vector store, RAG core, network runtime loop, or live LLM path is used by these commands.
