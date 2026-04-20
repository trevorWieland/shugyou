---
kind: standard
name: schema-versioning
category: rust-bevy-replay
importance: high
applies_to:
- 'crates/game-command/**/*.rs'
- 'crates/game-observation/**/*.rs'
- 'crates/game-replay/**/*.rs'
applies_to_languages:
- rust
applies_to_domains:
- schema
---

`GameCommand`, `Observation`, and replay payloads must carry explicit schema versions. Breaking contract changes require version increments.
