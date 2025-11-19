#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/web"
npm install
npm run build
cd "$ROOT"
cargo run --manifest-path core/Cargo.toml -- dashboard
