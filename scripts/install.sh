#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to install Preflight" >&2
  exit 1
fi
if ! command -v npm >/dev/null 2>&1; then
  echo "npm is required to build the dashboard" >&2
  exit 1
fi
if [ -f "Cargo.lock" ]; then
  cargo install --path core --locked
else
  cargo install --path core
fi
cd "$ROOT/web"
npm install
npm run build
cd "$ROOT"
echo "Preflight installed. Run 'preflight scan' then 'preflight dashboard'."
