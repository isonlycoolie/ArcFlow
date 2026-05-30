/** Graph workflow metadata compiled for the native binding. */

export interface GraphNodeSpec {
  id: string;
  stepId: string;
}

export interface GraphEdgeSpec {
  from: string;
  to?: string | null;
  condition?: string | null;
}

export interface GraphPayload {
  entry_node: string;
  max_iterations: number;
  nodes: Array<{ id: string; step_id: string }>;
  edges: Array<{ from: string; to?: string | null; condition?: string | null }>;
  join_nodes?: Array<{ id: string; wait_for: string[] }>;
}

export function buildGraphJson(options: {
  entryNode: string;
  maxIterations: number;
  nodes: GraphNodeSpec[];
  edges: GraphEdgeSpec[];
  joinNodes?: Array<{ id: string; waitFor: string[] }>;
}): string {
  const payload: GraphPayload = {
    entry_node: options.entryNode,
    max_iterations: options.maxIterations,
    nodes: options.nodes.map((n) => ({ id: n.id, step_id: n.stepId })),
    edges: options.edges.map((e) => ({
      from: e.from,
      to: e.to ?? null,
      condition: e.condition ?? null,
    })),
  };
  if (options.joinNodes?.length) {
    payload.join_nodes = options.joinNodes.map((j) => ({
      id: j.id,
      wait_for: j.waitFor,
    }));
  }
  return JSON.stringify(payload);
}
