/** RCS WorkflowDefinition subset used by the graph webview (snake_case JSON). */

export interface WorkflowDefinition {
  id: string;
  name: string;
  steps: StepDefinition[];
  execution_mode?: "linear" | "graph";
  graph?: GraphDefinition;
}

export interface StepDefinition {
  id: string;
  agent_id: string;
  order: number;
  fallback_step_id?: string;
}

export interface GraphDefinition {
  entry_node: string;
  max_iterations?: number;
  nodes: GraphNode[];
  edges: GraphEdge[];
  join_nodes?: JoinNode[];
}

export interface GraphNode {
  id: string;
  step_ref: string;
  inputs?: string[];
  outputs?: string[];
}

export interface GraphEdge {
  from: string;
  to?: string;
  condition?: string;
}

export interface JoinNode {
  id: string;
  wait_for: string[];
}

/** ExecutionTrace subset for trace timeline stub. */
export interface ExecutionTrace {
  run_id: string;
  workflow_name: string;
  status: string;
  started_at: string;
  completed_at?: string;
  duration_ms?: number;
  steps: StepTrace[];
  events_dropped?: number;
}

export interface StepTrace {
  step_index: number;
  step_id: string;
  agent_name: string;
  agent_role: string;
  status: string;
  started_at: string;
  completed_at?: string;
  duration_ms?: number;
}

export interface GraphLayoutNode {
  id: string;
  label: string;
  stepRef: string;
  x: number;
  y: number;
  isEntry: boolean;
  isJoin: boolean;
}

export interface GraphLayoutEdge {
  from: string;
  to: string;
  label?: string;
}

export interface GraphLayout {
  nodes: GraphLayoutNode[];
  edges: GraphLayoutEdge[];
  mode: "graph" | "linear";
  workflowName: string;
  warnings: string[];
}
