---
kind: standard
name: fixed-schedule-and-seeded-rng
category: rust-bevy-determinism
importance: critical
applies_to:
- 'crates/game-simulation/**/*.rs'
- 'bin/game-headless/**/*.rs'
applies_to_languages:
- rust
applies_to_domains:
- determinism
---

Simulation must advance using deterministic tick stepping and seeded RNG resources; never allocate unseeded randomness in state mutation paths.
