import React, { useEffect, useState } from 'react';
import { GraphView } from './GraphView';
import { IssuesPanel } from './IssuesPanel';
import { SystemState } from './types';
import './index.css';

const fetchState = async (): Promise<SystemState | null> => {
  try {
    const res = await fetch('/.preflight/scan.json', { cache: 'no-cache' });
    if (!res.ok) {
      console.error('Failed to fetch scan.json', res.statusText);
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
          <div style={{ color: '#94a3b8' }}>.preflight/scan.json is the single source of truth.</div>
        </div>
        <div className={badge.className}>{badge.text}</div>
      </header>
      <main>
        <GraphView state={state} />
        <IssuesPanel issues={state?.issues ?? []} />
      </main>
    </div>
  );
}
