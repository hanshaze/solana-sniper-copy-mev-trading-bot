#!/usr/bin/env bash
set -euo pipefail

# ─── 1) jump to script’s own dir ───
cd "$(dirname "$0")"

# ─── 2) build in release (skips if up-to-date) ───
cargo build --release

# ─── 3) where our binary really lives ───
BIN="$(pwd)/target/release/grpc-raydium-pool-monitoring-rust"

# ─── 4) if PIN_CORES is set, pin there; otherwise run normally ───
if [[ -n "${PIN_CORES:-}" ]]; then
  echo "📌 pinning to cores: $PIN_CORES"
  exec taskset -c "$PIN_CORES" "$BIN" "$@"
else
  echo "▶️ running on all available cores"
  exec "$BIN" "$@"
fi
