# W0 Observation Schema

Observation JSONL is the only W0 world-to-future-agent channel.

Each line is one JSON object:

```json
{
  "tick": 1,
  "map_seed": 7,
  "rule_seed": 3,
  "self": { "x": 4, "y": 3 },
  "energy_bucket": 10,
  "reward_delta": 0,
  "blocked": false,
  "held_shape_id": null,
  "visible_entities": [
    {
      "local_id": 1,
      "shape_id": 4,
      "color_id": 738,
      "size": 2,
      "x": 5,
      "y": 0,
      "dx": 1,
      "dy": -3,
      "adjacent": false,
      "distance_to_self": 4
    }
  ]
}
```

## Top-Level Fields

- `tick`: deterministic world tick after action application.
- `map_seed`: map initialization seed.
- `rule_seed`: hidden rule-table initialization seed.
- `self`: current position.
- `energy_bucket`: bounded integer state bucket.
- `reward_delta`: exact energy delta for the just-applied tick after clamping.
- `blocked`: whether the just-applied movement action was blocked.
- `held_shape_id`: neutral appearance id for the held entity, or `null`.
- `visible_entities`: all currently present W0 entities sorted by `local_id`.

## Visible Entity Fields

- `local_id`: stable local numeric id while that entity exists.
- `shape_id`: neutral numeric appearance id.
- `color_id`: neutral numeric appearance id.
- `size`: neutral numeric size bucket.
- `x`, `y`: entity position.
- `dx`, `dy`: entity offset from `self`.
- `adjacent`: Manhattan distance equals 1.
- `distance_to_self`: Manhattan distance from `self`.

## Explicitly Absent

Observation JSON must not contain:

- behavior class codes
- matching-table information
- open or locked state
- passability flags
- English object labels or world-semantic role names

The absence of a removed entity is the only observation-facing signal that a state change happened.
