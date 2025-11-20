# Preflight

## What Preflight Is
Preflight is a deterministic system scanner and dashboard feed. The Rust CLI inspects your host for Docker, Node.js, databases, GPUs, and common port conflicts, then writes a reproducible JSON contract under `.preflight/scan.json`. The bundled dashboard server (Axum + Vite build artifacts) reads the same contract to visualize nodes, edges, and issues without diverging logic.

## Installation
- Install the CLI from source:
  ```bash
  cargo install --path core --locked
  ```
- Build the dashboard bundle (served by `preflight dashboard`):
  ```bash
  cd web
  npm install
  npm run build
  ```
  Return to the repo root after the build so the CLI can find `web/dist`.

## Commands
All commands accept the global `--json` flag to emit structured envelopes for automation.

- `scan`: Collects system facts, derives graph edges, writes `.preflight/scan.json`, and records history under `.preflight/history`.
- `doctor`: Checks for required tools and summarizes known issues with fixability hints.
- `deps`: Produces a dependency graph of services discovered during scanning.
- `analyze`: Runs root-cause analysis over recorded issues and suggests fixes.
- `validate`: Verifies the current system state matches Preflight's internal contract invariants.
- `validate-env`: Loads `.preflight.yml` and compares environment expectations (Docker API/Compose, Node.js, GPU policy) against the latest scan.
- `security`: Performs security-oriented checks (ports, Docker, shell history) against the collected state.
- `export --format <mermaid|graphviz>`: Renders the latest graph in the chosen format to stdout for downstream visualization.
- `share --out <path.zip>`: Builds a portable bundle containing scan output, dependency graph, analysis results, validate-env results, and scan history.
- `upgrade`: Fetches the latest GitHub release tagged with a SemVer version, downloads the platform binary, and atomically replaces the current executable when a newer version exists.

Additional helpers include `dashboard` (serves the UI), `watch` (continuously scans), `snapshot` save/restore, and `fix` suggestions; they also honor `--json` where applicable.

## Dashboard Usage
Run `preflight dashboard` after performing a scan. The server serves static assets from `web/dist` and responds on `http://127.0.0.1:8787` with:
- `/api/state`: the live JSON contract derived from `.preflight/scan.json`.
- `/`: the React UI that renders the graph and issue list.

If the dashboard bundle is missing, the CLI will tell you to build it from the `web` directory.

## Environment-as-code (.preflight.yml)
Define expected Docker API/Compose versions, minimum Node.js versions, and GPU policy in `.preflight.yml`. Running `preflight validate-env` compares those requirements against the latest scan data, reporting violations both in the terminal and as `.preflight/validate_env.json` when JSON mode is enabled.

## Bundle Export Workflow
1. Run `preflight scan` to capture the current state.
2. Optionally run `preflight deps`, `preflight analyze`, and `preflight validate-env` to refresh derived artifacts.
3. Package everything with `preflight share --out preflight-bundle.zip`. The bundle includes the scan contract, dependency graph, analysis output, environment validation results, and scan history for transport or archival.

## Examples with JSON Output
- Scan with machine-readable output:
  ```bash
  preflight scan --json
  ```
- Validate environment expectations:
  ```bash
  preflight validate-env --json
  ```
- Perform an in-place upgrade when a newer SemVer tag exists:
  ```bash
  preflight upgrade --json
  ```

The JSON envelopes include the command name, status, timestamp, and data payloads to simplify scripting and dashboards.
