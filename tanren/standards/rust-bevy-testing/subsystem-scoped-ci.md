---
kind: standard
name: subsystem-scoped-ci
category: rust-bevy-testing
importance: medium
applies_to:
- '.github/workflows/*.yml'
- 'justfile'
applies_to_languages:
- rust
applies_to_domains:
- testing
---

CI should keep fast default gates while enabling targeted subsystem tests and nightly extended suites for determinism/replay/balance.
