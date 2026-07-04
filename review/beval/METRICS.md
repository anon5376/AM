# Track B Metrics V1

Status: frozen for B02. Post-hoc metric edits are treated as L15-class test weakening.

## Deterministic Scoring

Responses are scored only from a single line beginning with `ANSWER:`. Missing envelope is `NoAnswerEnvelope` and fails the task. The allowed deterministic matchers are:

- `exact`
- `regex`
- `must_contain_all`
- `must_contain_none`

No LLM-as-judge, embedding similarity, or human scoring is allowed.

## Categories

- `stage_accuracy`: exact or structured-match answers about current committed AM001 state.
- `stale_claim`: planted outdated claims repeated versus flagged. Lower stale repeat rate is better.
- `contradiction_handling`: open contradiction tasks requiring qualification or a request for verification. In B02 these are B2-only and skipped elsewhere without penalty.
- `test_grounded`: recommendations must cite current verified artifact names or values.
- `drift`: paired repeated questions; same state must produce the same structured answer.

## Metrics

Per-category accuracy:

```text
matched / total
```

where `total` excludes tasks skipped by lane.

Stale claim rate:

```text
stale_repeats / stale_claim_tasks_evaluated
```

Answer drift:

```text
mismatched_drift_groups / evaluated_drift_groups
```

Token cost:

```text
tokens = ceil(chars / 4)
mean_context_tokens = sum(context_tokens_per_evaluated_task) / evaluated_tasks
```

B1 context truncation is deterministic recency-window truncation: keep the tail of `beval/fixtures/rawlog_v1.md` within the 8,000-token budget.

## Overall Table

Reports must include:

```text
lane | evaluated | skipped | stage_accuracy | stale_claim_rate | contradiction_handling | test_grounded | answer_drift | mean_context_tokens
```

## B06 Gate

B2 must beat B1 with at least 5x fewer context tokens, or the finding is diagnosis: parameter memory is not yet beating text memory.
