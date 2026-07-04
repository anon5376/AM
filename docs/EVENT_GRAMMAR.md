# EG-1 Event Grammar

EG-1 is the Track B batch write format for `am apply`. It is line-oriented JSONL with a closed header and one event envelope per subsequent line.

## Header

The first line is required and must be exactly this grammar object, with no extra fields:

```json
{"grammar":"EG-1"}
```

Missing, malformed, or wrong-version headers reject the whole file before snapshot or staging sidecar load.

## Envelope

Every event line is a closed JSON object:

```json
{
  "id": 1,
  "session_id": "session_1",
  "source": "user",
  "verb": "assert",
  "args": {}
}
```

Fields:

- `id`: `u64`, strictly increasing within the file and within the core trace `i64` range.
- `session_id`: non-empty string.
- `source`: one of `user`, `test_verified`, `llm_claim`.
- `verb`: one of `assert`, `cue`, `link`, `goal_push`, `goal_pop`.
- `args`: verb-specific closed object.

Unknown fields, unknown enum values, non-monotonic ids, bad numeric values, unknown axes, and out-of-range strengths or weights are rejected with machine-readable reasons in the apply report.

## Verbs

`assert`:

```json
{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0}]}
```

`axes` must be non-empty. Each `axis` must exist in the AM axis registry. Values must be finite `f32`.

`cue`:

```json
{"concept":"rust","strength":0.8}
```

`strength` must be in `(0,1]`.

`link`:

```json
{"a":"rust","b":"am001","weight":1.0}
```

`weight` must be in `[-1,1]`.

`goal_push` and `goal_pop`:

```json
{"concept":"rust"}
```

These delegate to the existing core goal push/pop operations. B01 adds no new core goal mechanism.

## Apply Semantics

`am apply --snapshot <path> --events <file.jsonl> [--report <out.json>]` parses the whole file before mutation. Structural parse errors reject the file before snapshot mutation. Semantic per-event rejections skip that event; valid events still run in deterministic file order.

Events with source `user` or `test_verified` apply through the same `step_result` path used by `step-text`. Mutation records and step traces already carry `event_id`, so B01 does not change core trace schema. The CLI writes a deterministic trace file beside the snapshot:

```text
<snapshot>.trace.jsonl
```

The report contains per-event verdicts:

```json
{"id":1,"action":"applied","source":"user"}
```

Actions are `applied`, `staged`, `rejected`, `committed_from_staging`, and `expired`.

## Staging Sidecar

The staging sidecar is bridge-owned and lives beside the snapshot:

```text
<snapshot>.staging
```

It is versioned, sorted before serialization, and byte-stable for roundtrip hashing. The core snapshot format remains v4.

`llm_claim` events never touch AM state directly.

- `assert` claims are staged per `(concept, axis)` so corroborating one axis cannot commit an uncorroborated sibling axis from the same JSONL event.
- `link` claims are staged as `(a, b, sign(weight))`.
- `cue` claims are rejected outright with `CueClaimRejected`; cueing is attention steering, not memory evidence.
- `goal_push` and `goal_pop` claims are rejected with `UnimplementedVerb` because B01 defines no corroboration rule for goals.

Commit rule:

- A staged assert commits when a later `user` or `test_verified` event asserts the same `(concept, axis)` with the same value sign.
- A staged link commits when a later `user` or `test_verified` event links the same `(a, b)` pair with the same weight sign.
- The corroborating event applies normally first. The staged event then applies through the normal step path and is reported as `committed_from_staging`.

Expiry rule:

- Staged entries expire after 200 subsequently applied non-staged events.
- Expiry is reported and the entry is moved into the sidecar tombstone section.

Sidecar corruption or version mismatch refuses the apply run loudly. It is never silently reset.

## Anti-Laundering Rationale

The core's delta rule reinforces repetition. An unstaged LLM write path would let a model inflate its own confabulations into high-certainty parameter memory by re-asserting them. Staging plus corroboration makes LLM claims inert until reality through tests, or the user, co-signs them.
