export type NodeType =
  | 'os'
  | 'service'
  | 'runtime'
  | 'application'
  | 'port'
  | 'file'
  | 'python'
  | 'postgres'
  | 'mysql'
  | 'redis'
  | 'gpu'
  | 'dockerimages';
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
  risk_score_total: number;
  risk_issue_breakdown: [string, number][];
  version: string;
  timestamp: string;
  fingerprint: string;
}
