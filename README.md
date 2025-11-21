# Preflight

Preflight is a deterministic system scanner and dashboard. The Rust CLI inspects the host for Docker, runtimes, databases, GPUs, and common port conflicts, then writes a canonical JSON contract at `.preflight/scan.json`. The Axum-powered dashboard reads that same contract and serves the built Vite bundle from `/dashboard/` so the UI and CLI never diverge.

## Installation

1. Install the Rust CLI directly from the workspace:
   ```bash
   cargo install --path core --locked
   ```
2. Build the dashboard bundle so `preflight dashboard` can serve it:
   ```bash
   cd web
   npm install
   npm run build
   cd ..
   ```

## Running the scanner

Perform a local scan and write the canonical contract:
```bash
preflight scan
```

Useful flags:
- `--json` — wrap command output in a deterministic JSON envelope.
- `preflight scan --json` — emit scan results to stdout while also persisting `.preflight/scan.json`.

The scan pipeline detects:
- Docker daemon availability and Compose metadata.
- Node.js and npm versions plus dependency drift.
- Python versions and dependency drift across `requirements.txt`, Pipenv, and Poetry.
- Database availability for PostgreSQL, MySQL, and Redis (including open ports and running processes).
- GPU presence via `nvidia-smi`, `lspci`, CUDA, and cuDNN headers.
- Port conflicts for 3000, 5173, 8000, and 8080.

All nodes, edges, and issues are normalized and fingerprinted to ensure identical output on identical machines. JSON keys are alphabetized before writing.

## Dashboard

After building the dashboard bundle, serve it with:
```bash
preflight dashboard
```

The Axum server mounts the dashboard at `http://127.0.0.1:8787/dashboard/` and exposes:
- `GET /api/state` — the parsed `.preflight/scan.json` contract with risk scores.
- `GET /api/mtime` — the deterministic timestamp and fingerprint for cache busting.

The Vite build uses the `/dashboard/` base path, and all static assets are embedded from `web/dist`.

## Shared contract

The CLI and dashboard share `scan.schema.json` (schema version `1.0.0`). Every scan is validated against this schema at runtime, guaranteeing that the dashboard and any downstream tooling consume the same strict contract. The schema requires concrete typing for nodes, edges, issues, version, timestamp, and fingerprint.

## Determinism rules

- No wall-clock timestamps: timestamps are fixed, and a SHA-256 fingerprint summarizes the state.
- Maps use ordered serialization and are written with alphabetized JSON keys for reproducible diffs.
- Fingerprints are recomputed after graph derivation and issue evaluation to stabilize repeated runs.

## Release artifacts

GitHub Actions builds run `cargo build --release` and `cargo test --all`, checksum the resulting binary, and upload artifacts. A matrix build publishes platform-specific binaries to `dist/` for:
- Linux x86_64
- macOS x86_64
- macOS ARM64
- Windows x86_64

