#!/usr/bin/env bash
set -euo pipefail
preflight scan
cat .preflight/scan.json
