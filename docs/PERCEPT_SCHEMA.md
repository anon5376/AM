# Percept Schema

The P03 bridge consumes W0 observation JSON and emits AM `Event`s. Observation JSON remains the only world-to-agent channel.

## Percept

```text
Percept {
  local_id: u32,
  feat: [f32; 12],
  pose: Pose
}
```

`feat` is appearance only:

- indexes `0..5`: five shape one-hots, slot `(shape_id - 1) mod 5`
- indexes `5..11`: six color one-hots, slot `color_id mod 6`
- index `11`: size scalar, `size / 3.0`

Pose is track/state data, not appearance:

```text
Pose { x, y, dx, dy, adjacent, distance_to_self }
```

Pose is never written into the appearance vector.

## Event Emission

For each visible entity:

- cue `loc_<local_id>` with strength `0.8`
- assert existing AM axes:
  - `temporal_near=1`
  - `concreteness=1`
  - `novelty=η`
- if adjacent, link `trk_0 loc_<local_id> 0.6`

`trk_0` is the temporary self row label. `loc_<local_id>` labels are temporary P03 labels and are flagged for replacement by S04-style `trk_<n>` track labels.

Novelty is specified as `1 - max cosine(feat, existing prototypes in P)`. P is not present in this session, so the bridge uses a session-local feature-key set:

- unseen local feature key: `novelty=1`
- seen local feature key: `novelty=0`

This is temporary and must be replaced when the P store lands.

The bridge emits no causal relation, no schema, and no world-mechanic field. Every emitted `Event` is passed through the existing AM validator before use.
