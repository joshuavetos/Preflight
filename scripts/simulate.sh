#!/usr/bin/env bash
set -euo pipefail
if [ "$#" -lt 1 ]; then
  echo "usage: $0 \"<command>\"" >&2
  exit 1
fi
preflight simulate "$*"
