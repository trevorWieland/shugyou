#!/usr/bin/env bash
set -euo pipefail

out1=$(mktemp)
out2=$(mktemp)
trap 'rm -f "$out1" "$out2"' EXIT

cargo run -p game-headless -- --ticks 60 --seed 42 >"$out1"
cargo run -p game-headless -- --ticks 60 --seed 42 >"$out2"

diff -u "$out1" "$out2" >/dev/null
echo "determinism smoke check passed"
