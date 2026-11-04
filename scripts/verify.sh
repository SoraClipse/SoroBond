#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mode="${1:-all}"
if [[ "$mode" == "js" || "$mode" == "all" ]]; then
  # This repository has no package.json, pnpm lockfile, or TypeScript sources.
  # Do not report an invented pnpm install, lint or typecheck.
  while IFS= read -r -d '' file; do
    node --check "$file"
  done < <(find apps packages tests scripts -type f -name '*.mjs' -print0)
  node --test packages/sdk/test/*.test.mjs apps/api/test/*.test.mjs apps/web/tests/*.test.mjs tests/e2e/*.test.mjs
  node scripts/check-docs.mjs
fi
if [[ "$mode" == "rust" || "$mode" == "all" ]]; then
  cargo fmt --manifest-path contracts/Cargo.toml --check
  cargo test --manifest-path contracts/Cargo.toml
  cargo clippy --manifest-path contracts/Cargo.toml --all-targets -- -D warnings
fi
if [[ "$mode" != "js" && "$mode" != "rust" && "$mode" != "all" ]]; then
  echo 'Usage: bash scripts/verify.sh [js|rust|all]' >&2
  exit 2
fi
git diff --check
