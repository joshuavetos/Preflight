import React, { useEffect, useState } from 'react';
import { GraphView } from './GraphView';
import { IssuesPanel } from './IssuesPanel';
import { RiskPanel } from './RiskPanel';
import { SystemState } from './types';
import './index.css';

const fetchState = async (): Promise<SystemState | null> => {
  try {
    const res = await fetch('/api/state', { cache: 'no-cache' });
    if (!res.ok) {
      const message = await res.text();
      console.error('Failed to fetch state', res.status, message);
      return null;
    }
    const json = (await res.json()) as SystemState;
    return json;
  } catch (err) {
    console.error('Fetch error', err);
    return null;
  }
};

function statusLabel(state: SystemState | null) {
  if (!state || state.issues.length === 0) {
    return { text: 'Ready for Takeoff', className: 'status ok' };
  }
  return { text: 'Flight Risk Detected', className: 'status warn' };
}

export default function App() {
  const [state, setState] = useState<SystemState | null>(null);

  useEffect(() => {
    fetchState().then(setState);
  }, []);

  const badge = statusLabel(state);

  return (
    <div>
      <header>
        <div>
          <h1>Preflight Dashboard</h1>
          <div style={{ color: '#94a3b8' }}>Serving data from the Rust API at /api/state.</div>
          {state ? (
            <div style={{ color: '#94a3b8', fontSize: '0.9rem' }}>
              Version {state.version} · Captured at {state.timestamp}
            </div>
          ) : null}
        </div>
        <div className={badge.className}>{badge.text}</div>
      </header>
      <main>
        <div className="graph-column">
          <GraphView state={state} />
        </div>
        <div className="sidebar">
          {state ? (
            <RiskPanel
              score={state.risk_score_total}
              breakdown={state.risk_issue_breakdown}
            />
          ) : null}
          <IssuesPanel issues={state?.issues ?? []} />
        </div>
      </main>
    </div>
  );
}
