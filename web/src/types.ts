export type NodeType = 'OS' | 'Service' | 'Runtime' | 'Application' | 'Port' | 'File';
export type Status = 'active' | 'inactive' | 'conflict';
export type Relation = 'REQUIRES' | 'BINDS' | 'CONFLICTS';
export type Severity = 'critical' | 'warning';

export interface Node {
  id: string;
  type: NodeType;
  label: string;
  status: Status;
  metadata: Record<string, string>;
}

export interface Edge {
  from: string;
  to: string;
  relation: Relation;
}

export interface Issue {
  id: number;
  severity: Severity;
  title: string;
  description: string;
  suggestion: string;
}

export interface SystemState {
  nodes: Node[];
  edges: Edge[];
  issues: Issue[];
}
