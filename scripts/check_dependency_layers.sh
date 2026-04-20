#!/usr/bin/env bash
set -euo pipefail

fail=0

check_forbidden_dep() {
  local file="$1"
  local dep="$2"
  local label="$3"
  if rg -n "^${dep}\\s*=|^${dep}\\.workspace\\s*=\s*true" "$file" >/dev/null; then
    echo "forbidden dependency in ${label}: ${dep}"
    fail=1
  fi
}

# Domain rule: no workspace path deps.
if rg -n 'path\s*=\s*"\.\./' crates/game-domain/Cargo.toml >/dev/null; then
  echo "forbidden workspace dependency in game-domain"
  fail=1
fi

# Command rule: only game-domain from workspace crates.
for dep in game-observation game-content game-simulation game-presentation game-input game-net game-mcp game-mod-api game-testing game-platform game-replay; do
  check_forbidden_dep crates/game-command/Cargo.toml "$dep" "game-command"
done

# Simulation rule: never depend on presentation/input/net.
for dep in game-presentation game-input game-net game-mcp; do
  check_forbidden_dep crates/game-simulation/Cargo.toml "$dep" "game-simulation"
done

# Headless and server binaries must not include presentation or full bevy umbrella.
for file in bin/game-headless/Cargo.toml bin/game-server/Cargo.toml; do
  check_forbidden_dep "$file" "game-presentation" "$file"
  check_forbidden_dep "$file" "bevy" "$file"
done

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi

echo "dependency layer checks passed"
