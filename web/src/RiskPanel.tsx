import React from 'react';

interface Props {
  score: number;
  breakdown: [string, number][];
}

export const RiskPanel: React.FC<Props> = ({ score, breakdown }) => {
  const level =
    score >= 80 ? 'critical' :
    score >= 50 ? 'warning' :
    'ok';

  return (
    <div className="panel">
      <h2>Risk Assessment</h2>
      <div className={`risk-score risk-${level}`}>
        Overall Risk Score: {score}
      </div>

      <div style={{ marginTop: '1rem', fontWeight: 600 }}>
        Breakdown
      </div>

      <ul style={{ marginTop: '0.5rem' }}>
        {breakdown.map(([code, val]) => (
          <li key={code} style={{ marginBottom: '0.25rem' }}>
            <span style={{ fontWeight: 600 }}>{code}:</span> {val}
          </li>
        ))}
      </ul>
    </div>
  );
};
