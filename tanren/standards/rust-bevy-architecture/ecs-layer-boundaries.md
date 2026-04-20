---
kind: standard
name: ecs-layer-boundaries
category: rust-bevy-architecture
importance: critical
applies_to:
- 'crates/**/*.rs'
applies_to_languages:
- rust
applies_to_domains:
- architecture
---

Honor crate-layer boundaries: domain -> command/observation/content -> simulation -> presentation/input/net adapters. New dependencies must preserve this graph.
