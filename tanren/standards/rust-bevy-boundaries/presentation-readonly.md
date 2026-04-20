---
kind: standard
name: presentation-readonly
category: rust-bevy-boundaries
importance: high
applies_to:
- 'crates/game-presentation/**/*.rs'
- 'bin/game/**/*.rs'
applies_to_languages:
- rust
applies_to_domains:
- boundaries
---

Presentation systems read observation/simulation outputs but do not own authoritative state mutation logic.
