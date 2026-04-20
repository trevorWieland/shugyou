#!/usr/bin/env bash
set -euo pipefail

if rg -n --glob '*.rs' '#\[allow\(' crates bin xtask 2>/dev/null; then
  echo "inline #[allow(...)] is disallowed"
  exit 1
fi

if rg -n --glob '*.rs' '#\[allow\(clippy::' crates bin xtask 2>/dev/null; then
  echo "inline clippy allow is disallowed"
  exit 1
fi

echo "no inline suppressions detected"
