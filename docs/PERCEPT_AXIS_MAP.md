# Percept Axis Map

P03 maps observation-derived signals only onto existing AM axes.

## Entity Rows

Temporary label:

```text
loc_<local_id>
```

Axes:

```text
temporal_near = 1.0
concreteness  = 1.0
novelty       = eta
```

`eta` is the temporary session-local novelty value described in `docs/PERCEPT_SCHEMA.md`.

## Self Row

Temporary label:

```text
trk_0
```

Axes:

```text
temporal_near          = 1.0
value                  = clamp(energy_bucket / 20.0, 0.0, 1.0)
risk                   = 1.0 - value
constraint_relevance   = 1.0 only when blocked=true
```

The energy constants mirror the current W0 default bucket range. If `WorldTheta.max_energy_bucket` becomes variable at the bridge boundary, this map must move to a theta-style percept config.

## Links

Adjacent visible entities produce only:

```text
link trk_0 loc_<local_id> 0.6
```

No other relation type is emitted by perception in P03.
