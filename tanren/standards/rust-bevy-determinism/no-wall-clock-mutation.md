---
kind: standard
name: no-wall-clock-mutation
category: rust-bevy-determinism
importance: critical
applies_to:
- 'crates/game-simulation/**/*.rs'
applies_to_languages:
- rust
applies_to_domains:
- determinism
---

No wall-clock reads (`Instant`, `SystemTime`, frame delta) are allowed in simulation mutation systems. Simulation time comes from fixed tick counters.
