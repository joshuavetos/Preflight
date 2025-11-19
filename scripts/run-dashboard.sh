#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../web"
npm install
npm run dev -- --host 0.0.0.0 --port 5173
