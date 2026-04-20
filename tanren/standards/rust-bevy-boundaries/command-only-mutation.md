---
kind: standard
name: command-only-mutation
category: rust-bevy-boundaries
importance: critical
applies_to:
- 'crates/**/*.rs'
applies_to_languages:
- rust
applies_to_domains:
- boundaries
---

Simulation state mutations must be triggered only by typed `GameCommand` flow through the command buffer/application path.
