import React from 'react';
import { Issue } from './types';

interface Props {
  issues: Issue[];
}

export const IssuesPanel: React.FC<Props> = ({ issues }) => {
  if (!issues.length) {
    return <div className="panel">No issues detected. Ready for takeoff.</div>;
  }
  return (
    <div className="panel">
      <h2>Issues</h2>
      <ul>
        {issues.map((issue) => (
          <li key={issue.code} style={{ marginBottom: '0.75rem' }}>
            <div style={{ fontWeight: 700 }}>
              [{issue.severity.toUpperCase()}] {issue.title} ({issue.code})
            </div>
            <div style={{ color: '#cbd5e1' }}>{issue.description}</div>
            <div style={{ color: '#a3e635' }}>Suggestion: {issue.suggestion}</div>
          </li>
        ))}
      </ul>
    </div>
  );
};
