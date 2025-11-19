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

export const GraphView: React.FC<Props> = ({ state }) => {
  if (!state) {
    return <div className="panel">No scan data available.</div>;
  }

  const nodes: Node[] = state.nodes.map((n, index) => ({
    id: n.id,
    position: { x: (index % 3) * 200, y: Math.floor(index / 3) * 150 },
    data: { label: `${n.label} (${n.type.toUpperCase()})` },
    style: {
      border: `2px solid ${statusColor(n.status)}`,
      background: '#111827',
      color: '#e2e8f0',
    },
  }));

  const edges: Edge[] = state.edges.map((e) => ({
    id: `${e.from}-${e.to}-${e.relation}`,
    source: e.from,
    target: e.to,
    label: e.relation,
    style: { stroke: '#94a3b8' },
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
