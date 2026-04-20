#!/usr/bin/env bash
set -euo pipefail

repo="${1:-}"
if [[ -z "$repo" ]]; then
  repo=$(git remote get-url origin 2>/dev/null | sed -E 's#(git@github.com:|https://github.com/)##; s#\.git$##') || true
fi

if [[ -z "$repo" ]]; then
  echo "usage: $0 <owner/repo>"
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI is required"
  exit 1
fi

gh repo edit "$repo" \
  --description "Shugyou: Rust + Bevy command/observation-first game architecture" \
  --homepage "https://github.com/$repo"

gh repo edit "$repo" --add-topic rust --add-topic bevy --add-topic game-dev --add-topic deterministic-simulation --add-topic mcp

ruleset_payload=$(mktemp)
trap 'rm -f "$ruleset_payload"' EXIT

cp .github/rulesets/main.json "$ruleset_payload"
gh api --method POST "/repos/$repo/rulesets" --input "$ruleset_payload"

echo "applied repository metadata and ruleset to $repo"
