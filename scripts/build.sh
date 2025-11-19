#!/usr/bin/env bash
set -euo pipefail
cargo build --workspace
cd web
npm install
npm run build
