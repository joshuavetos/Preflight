import React from 'react';
import ReactFlow, { Background, Controls, Edge, Node } from 'reactflow';
import 'reactflow/dist/style.css';
import { SystemState } from './types';

interface Props {
  state: SystemState | null;
}

const statusColor = (status: string) => {
  switch (status) {
    case 'active':
      return '#10b981';
    case 'conflict':
      return '#f97316';
    default:
      return '#6b7280';
  }
};

const typeLevel = (type: string) => {
  if (type === 'os') return 0;
  if (['service', 'postgres', 'redis', 'dockerimages'].includes(type)) return 150;
  if (['runtime', 'python', 'gpu', 'nodejs'].includes(type)) return 300;
  if (type === 'port') return 450;
  return 450;
};

export const GraphView: React.FC<Props> = ({ state }) => {
  if (!state) {
    return <div className="panel">No scan data available.</div>;
  }

  // Map issue severity to node risk
  const riskMap: Record<string, number> = {};
  state.issues.forEach((issue) => {
    const score = Number(issue.title.replace('Overall risk score: ', '').trim());
    if (!Number.isNaN(score)) {
      state.nodes.forEach((n) => (riskMap[n.id] = score));
    }
  });

  const level = (score: number | undefined) => {
    if (!score) return 'none';
    if (score >= 70) return 'critical';
    if (score >= 30) return 'warning';
    return 'low';
  };

  const highlightedNodes = new Set(
    state.nodes.filter((node) => node.status === 'conflict').map((node) => node.id)
  );

  const nodes: Node[] = state.nodes.map((n) => {
    const risk = level(riskMap[n.id]);

    return {
      id: n.id,
      // Safer layout: OS top, Services middle, Ports bottom
      position: {
        x: risk === 'critical' ? 200 : 150,
        y: typeLevel(n.type),
      },
      data: { label: `${n.label} (${n.type.toUpperCase()})` },
      style: {
        border: `2px solid ${statusColor(n.status)}`,
        background: '#111827',
        color: '#e2e8f0',
        boxShadow: highlightedNodes.has(n.id)
          ? '0 0 0 4px rgba(249, 115, 22, 0.25)'
          : risk === 'critical'
            ? '0 0 14px #ff4d4d'
            : risk === 'warning'
              ? '0 0 10px #f59e0b'
              : 'none',
        animation: risk === 'critical' ? 'pulseCritical 1.2s infinite' : 'none',
      },
    };
  });

  const edges: Edge[] = state.edges.map((e) => ({
    id: `${e.from}-${e.to}-${e.relation}`,
    source: e.from,
    target: e.to,
    label: e.relation,
    style: {
      stroke: highlightedNodes.has(e.from) || highlightedNodes.has(e.to)
        ? '#f97316'
        : '#94a3b8',
    },
    labelStyle: { fill: '#cbd5e1', fontWeight: 600 },
  }));

  return (
    <div className="panel" style={{ height: 500 }}>
      <ReactFlow nodes={nodes} edges={edges} fitView>
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
};
