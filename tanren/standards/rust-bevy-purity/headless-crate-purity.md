---
kind: standard
name: headless-crate-purity
category: rust-bevy-purity
importance: critical
applies_to:
- 'crates/game-simulation/**/*.rs'
- 'bin/game-headless/**/*.rs'
- 'bin/game-server/**/*.rs'
applies_to_languages:
- rust
applies_to_domains:
- dependency-purity
---

Headless and server execution paths must not depend on full rendering stacks. Keep simulation and server crates free of client rendering dependencies.
