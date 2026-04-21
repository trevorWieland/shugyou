set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just check

# Preferred entrypoint: repository utilities are invoked through `just`.
bootstrap:
  just tanren-bootstrap
  just tanren-install

setup:
  just bootstrap
  just check

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

lint:
  cargo clippy --workspace --all-targets -- -D warnings

unit:
  cargo test --workspace

nextest:
  @if command -v cargo-nextest >/dev/null 2>&1; then \
    cargo nextest run --workspace; \
  else \
    echo "cargo-nextest not found, falling back to cargo test --workspace"; \
    cargo test --workspace; \
  fi

doc:
  cargo test --workspace --doc

deny:
  @if command -v cargo-deny >/dev/null 2>&1; then \
    cargo deny check; \
  else \
    echo "cargo-deny not installed, skipping"; \
  fi

machete:
  @if command -v cargo-machete >/dev/null 2>&1; then \
    cargo machete; \
  else \
    echo "cargo-machete not installed, skipping"; \
  fi

quality:
  ./scripts/check_no_inline_suppressions.sh
  ./scripts/check_line_length.sh
  ./scripts/check_dependency_layers.sh

check:
  just fmt-check
  cargo check --workspace
  just lint
  just quality

ci:
  just check
  just nextest
  just doc
  just deny
  just machete

run-game:
  cargo run -p game

run-headless ticks='60' seed='42':
  cargo run -p game-headless -- --ticks {{ticks}} --seed {{seed}}

run-server vector='32':
  cargo run -p game-server -- --vector {{vector}}

run-mcp:
  cargo run -p game-mcp-bin -- --once

run-balance episodes='10' ticks='120' seed='42':
  cargo run -p game-balance -- --episodes {{episodes}} --ticks {{ticks}} --seed {{seed}}

nightly-determinism:
  ./scripts/check_determinism_smoke.sh

nightly-replay:
  cargo test -p game-replay

nightly-balance:
  cargo run -p game-balance -- --episodes 10 --ticks 120 --seed 42

tanren-bootstrap:
  cargo install --path ../tanren/bin/tanren-cli --locked --root .tooling

tanren-install:
  @if [[ -x .tooling/bin/tanren ]]; then \
    ./.tooling/bin/tanren install --profile rust-bevy; \
  elif command -v tanren >/dev/null 2>&1; then \
    tanren install --profile rust-bevy; \
  else \
    cargo run --manifest-path ../tanren/bin/tanren-cli/Cargo.toml -- install --profile rust-bevy; \
  fi

tanren-verify:
  @if [[ -x .tooling/bin/tanren ]]; then \
    ./.tooling/bin/tanren install --profile rust-bevy --dry-run --strict; \
  elif command -v tanren >/dev/null 2>&1; then \
    tanren install --profile rust-bevy --dry-run --strict; \
  else \
    cargo run --manifest-path ../tanren/bin/tanren-cli/Cargo.toml -- install --profile rust-bevy --dry-run --strict; \
  fi

github-apply repo='':
  @if [[ -n "{{repo}}" ]]; then \
    ./scripts/github/apply_repo_governance.sh "{{repo}}"; \
  else \
    ./scripts/github/apply_repo_governance.sh; \
  fi
