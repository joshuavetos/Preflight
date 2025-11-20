# Preflight

Preflight delivers a reproducible system scan, simulation engine, and Rust-hosted dashboard that all share the same JSON contract. The CLI is written in Rust, the dashboard is React + Vite, and the shared data is stored in `.preflight/scan.json`.

## Features
- Cross-platform CLI (Linux, macOS, Windows) with `scan`, `simulate`, and `dashboard` commands.
- Deterministic JSON contract with nodes, edges, issues, version, and timestamp fields.
- Rust dashboard server that serves the built React assets and exposes `/api/state`.
- Simulation engine that predicts port and Docker Compose conflicts before you run a command.
- Extended detectors for Docker, Python, Node.js, PostgreSQL, Redis, GPU availability, and common port checks.
- POSIX and PowerShell scripts for build, install, simulation, and dashboard launch.

## Install

### One-shot installer
- **POSIX:** `./scripts/install.sh`
- **PowerShell:** `./scripts/install.ps1`

Both installers require `cargo` and `npm` to be available. They build the CLI and the dashboard bundle.

### Manual steps
1. Build and install the CLI
   ```bash
   cargo install --path core --locked
   ```
2. Build the dashboard bundle
   ```bash
   cd web
   npm install
   npm run build
   cd ..
   ```

## Usage
- Scan the host and emit the contract
  ```bash
  preflight scan
  ```
- Simulate a command for conflicts
  ```bash
  preflight simulate "docker compose up -p demo"
  ```
- Launch the dashboard server (opens browser to http://127.0.0.1:8787)
  ```bash
  preflight dashboard
  ```

### Script shortcuts
- POSIX: `scripts/build.sh`, `scripts/run-dashboard.sh`, `scripts/simulate.sh`, `scripts/generate-graph-json.sh`
- PowerShell: `scripts/build.ps1`, `scripts/run-dashboard.ps1`, `scripts/simulate.ps1`, `scripts/generate-graph-json.ps1`

## API Contract
`/api/state` and `.preflight/scan.json` both conform to this schema:
```json
{
  "nodes": [{ "id": "os", "type": "os", "label": "...", "status": "active", "metadata": {} }],
  "edges": [{ "from": "docker", "to": "port8000", "relation": "BINDS" }],
  "issues": [{ "code": "DOCKER_INACTIVE", "severity": "warning", "title": "...", "description": "...", "suggestion": "..." }],
  "version": "1.0.0",
  "timestamp": "2024-01-01T00:00:00Z"
}
```
Keys are stable and casing is deterministic to simplify downstream consumers.

## Dashboard
- The Rust server serves static assets from `web/dist` and exposes `/api/state`.
- If the dashboard bundle is missing, the CLI will instruct you to run `npm install && npm run build` inside `/web`.
- The React app only fetches data via `/api/state`; it never reads the file system directly.

### ASCII Screenshot
```
+---------------------------------------------+
| Preflight Dashboard                         |
| Version 1.0.0 · Captured at 2024-01-01 ...  |
| [OK] Ready for Takeoff                      |
| Nodes/Edges graph (React Flow)              |
| Issues: [WARNING] Docker daemon inactive    |
+---------------------------------------------+
```

## Architecture (ASCII)
```
+-------------+       writes        +-----------------------+
| Rust CLI    | ------------------> | .preflight/scan.json  |
| (scan/sim)  |                    | (contract)            |
+-------------+                     +-----------------------+
        |                                    |
        | serves /api/state                  |
        v                                    v
+----------------------+          +------------------------+
| Rust Dashboard (axum)|          | React/Vite Dashboard   |
| static + JSON API    | <------> | fetches /api/state     |
+----------------------+          +------------------------+
```

## Governance Metadata
- auditor: Tessrax Governance Kernel v16
- clauses: AEP-001, RVC-001, EAC-001, POST-AUDIT-001, DLK-001, TESST

## License
MIT License. See [LICENSE](LICENSE).
