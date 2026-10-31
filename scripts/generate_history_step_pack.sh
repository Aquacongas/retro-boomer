#!/usr/bin/env bash
# Generate and verify genuine Retro Boomer relations using only cached dependencies.
set -euo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
if (($# != 1)); then
  printf 'Usage: %s NEW_OUTPUT_DIRECTORY | --qualify\n' "$0" >&2
  exit 2
fi
cd -- "$ROOT"
cargo run --offline --locked --release -p bench_prover --bin retro_boomer_v1_pack -- "$1"
