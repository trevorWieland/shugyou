#!/usr/bin/env bash
set -euo pipefail

max=120
violations=$(awk -v m="$max" 'length($0) > m {printf "%s:%d:%d\n", FILENAME, NR, length($0)}' $(rg --files -g '*.rs' crates bin xtask))

if [[ -n "${violations}" ]]; then
  echo "line length violations (> ${max}):"
  echo "${violations}"
  exit 1
fi

echo "line length check passed"
