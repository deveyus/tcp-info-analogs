#!/usr/bin/env bash
# Run the deterministic iai-callgrind instruction-count benchmarks.
#
# iai-callgrind measures machine-independent instruction counts (Ir),
# which gate decode-path regressions without wall-clock noise. The
# companion `iai-callgrind-runner` binary must be on PATH (cargo-
# installed, version pinned to match the lib in Cargo.toml). It lives
# at ~/.cargo/bin; nix-shell strips PATH, so PATH is re-exported
# inside the shell below.
#
# Saved baselines live under target/iai/ (names allow only [A-Za-z0-9_]).
#
# Usage:
#   ./scripts/bench_iai.sh                            # plain run, prints Ir
#   ./scripts/bench_iai.sh -- --save-baseline=decode_v1   # record a baseline
#   ./scripts/bench_iai.sh -- --baseline=decode_v1        # compare to it
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v iai-callgrind-runner >/dev/null 2>&1; then
    echo "iai-callgrind-runner not found on PATH." >&2
    echo "Install it with: cargo install iai-callgrind-runner --version 0.16.1" >&2
    exit 1
fi

nix-shell -p cargo rustc valgrind --run "cargo bench -p tcp-info-benches --bench decode $*"
