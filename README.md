# Preflight

Preflight delivers a reproducible system scan, simulation, and dashboard that all share the same data contract. The CLI is written in Rust and produces a deterministic JSON graph that the React dashboard consumes.

## Components
- **core**: Rust CLI providing `preflight scan`, `preflight simulate`, and `preflight dashboard` commands.
- **web**: Vite + React + TypeScript dashboard rendering the scan graph and issues.
- **.preflight/scan.json**: Canonical data contract shared between the CLI and the dashboard.

## Quickstart

```bash
# Build and install the CLI
cargo install --path core

# Run a scan and inspect the summary
preflight scan

# Simulate a workload command
preflight simulate "docker compose up"

# Launch the dashboard
cd web
npm install
npm run dev
```

## Data Contract
The scanner writes `.preflight/scan.json` with `nodes`, `edges`, and `issues` collections. Each execution overwrites the file atomically to avoid partial writes.

## Governance Metadata
- auditor: Tessrax Governance Kernel v16
- clauses: AEP-001, RVC-001, EAC-001, POST-AUDIT-001, DLK-001, TESST
