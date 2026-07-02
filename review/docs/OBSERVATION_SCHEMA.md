# W0 Observation Schema

Observation JSONL is the only W0 world-to-future-agent channel.

Each line is one JSON object:

```json
{
  "tick": 1,
  "map_seed": 7,
  "rule_seed": 3,
  "self": { "x": 3, "y": 2 },
  "energy_bucket": 5,
  "reward_delta": 0,
  "blocked": false,
  "visible_entities": [
    {
      "local_id": 1,
      "shape_id": 1,
      "color_id": 101,
      "size": 1,
      "x": 5,
      "y": 0,
      "dx": 2,
      "dy": -2,
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
- `reward_delta`: reward delta for the just-applied action.
- `blocked`: whether the just-applied action was blocked.
- `visible_entities`: all W0 entities sorted by `local_id`.

## Visible Entity Fields

- `local_id`: stable local numeric id.
- `shape_id`: neutral numeric appearance id.
- `color_id`: neutral numeric appearance id.
- `size`: neutral numeric size bucket.
- `x`, `y`: entity position.
- `dx`, `dy`: entity offset from `self`.
- `adjacent`: Manhattan distance equals 1.
- `distance_to_self`: Manhattan distance from `self`.

## Quarantine

Observation JSON must not contain world-semantic role names or English object labels. Shape, color, and size are neutral ids. Hidden mechanics remain internal to the world engine and do not appear in observation JSON.
