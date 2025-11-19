export type NodeType = 'os' | 'service' | 'runtime' | 'application' | 'port' | 'file';
export type Status = 'active' | 'inactive' | 'conflict';
export type Relation = 'REQUIRES' | 'BINDS' | 'CONFLICTS';
export type Severity = 'critical' | 'warning';

export interface Node {
  id: string;
  type: NodeType;
  label: string;
  status: Status;
  metadata: Record<string, unknown>;
}

export interface Edge {
  from: string;
  to: string;
  relation: Relation;
}

export interface Issue {
  code: string;
  severity: Severity;
  title: string;
  description: string;
  suggestion: string;
}

export interface SystemState {
  nodes: Node[];
  edges: Edge[];
  issues: Issue[];
  version: string;
  timestamp: string;
}
