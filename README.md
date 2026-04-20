# shugyou

Day 0 scaffold for a Rust/Bevy game workspace with Tanren methodology wiring.

## Preferred workflow

Use `just` as the primary interface for repo operations (setup, quality gates, runtime smoke commands, and governance utilities).

## Quick start

```bash
just setup
just check
just ci
just run-headless
just run-mcp
```

## Workspace shape

- `crates/`: domain, command/observation, simulation, replay, adapters
- `bin/`: client (`game`), server, MCP, headless, balance
- `tanren/standards`: project-owned `rust-bevy` standards

## GitHub governance

- CI and nightly workflows in `.github/workflows/`
- ruleset payload in `.github/rulesets/main.json`
- apply via justfile: `just github-apply owner/repo`
