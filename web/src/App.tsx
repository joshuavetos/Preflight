import React, { useEffect, useState } from 'react';
import { GraphView } from './GraphView';
import { IssuesPanel } from './IssuesPanel';
import { RiskPanel } from './RiskPanel';
import { SystemState } from './types';
import './index.css';

function statusLabel(state: SystemState | null) {
  if (!state || state.issues.length === 0) {
    return { text: 'Ready for Takeoff', className: 'status ok' };
  }
  return { text: 'Flight Risk Detected', className: 'status warn' };
}

export default function App() {
  const [state, setState] = useState<SystemState | null>(null);
  const [previousEtag, setPreviousEtag] = useState<string | null>(null);
  const [fadeClass, setFadeClass] = useState("");
  const [lastModified, setLastModified] = useState<string | null>(null);

  useEffect(() => {
    const run = async () => {
      try {
        const res = await fetch('/api/state', {
          cache: 'no-cache',
          headers: previousEtag ? { 'If-None-Match': previousEtag } : {},
        });

        if (res.status === 304) return; // No change

        if (!res.ok) {
          const message = await res.text();
          console.error('Failed to fetch state', res.status, message);
          return;
        }

        const newEtag = res.headers.get('ETag');
        const newState = (await res.json()) as SystemState;

        setFadeClass('fade-out');
        setTimeout(() => {
          setState(newState);
          setFadeClass('fade-in');
        }, 350);

        setLastModified(newState.timestamp);
        if (newEtag) setPreviousEtag(newEtag);
      } catch (err) {
        console.error('Fetch error', err);
      }
    };
    run();

    const id = setInterval(async () => {
      try {
        const res = await fetch('/api/mtime', { cache: 'no-cache' });
        if (!res.ok) return;
        const body = (await res.json()) as { timestamp?: string };
        if (body.timestamp && body.timestamp !== lastModified) {
          setLastModified(body.timestamp);
          await run();
        }
      } catch (err) {
        console.error('poll error', err);
      }
    }, 2000);

    return () => clearInterval(id);
  }, [previousEtag, lastModified]);

  const badge = statusLabel(state);

  return (
    <div className={fadeClass}>
      <header>
        <div>
          <h1>Preflight Dashboard</h1>
          <div style={{ color: '#94a3b8' }}>Serving data from the Rust API at /api/state.</div>
          {state ? (
            <div style={{ color: '#94a3b8', fontSize: '0.9rem' }}>
              Version {state.version} · Captured at {state.timestamp}
              <br />
              <span
                className={
                  state.issues.length === 0
                    ? 'risk-banner risk-low'
                    : state.issues.some((i) => i.severity === 'critical')
                      ? 'risk-banner risk-high'
                      : 'risk-banner risk-med'
                }
              >
                Risk Score:{' '}
                {state.issues.find((i) => i.code === 'SIM_RISK_SUMMARY')?.title
                  .replace('Overall risk score: ', '') ?? 0}
              </span>
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
