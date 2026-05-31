export interface GraphNodeSpec {
  id: string;
  stepRef: string;
  inputs?: string[];
  outputs?: string[];
}

export interface GraphEdgeSpec {
  from: string;
  to?: string | null;
  condition?: string | null;
}

export interface GraphPayload {
  entry_node: string;
  max_iterations: number;
  nodes: Array<{ id: string; step_ref: string; inputs?: string[]; outputs?: string[] }>;
  edges: Array<{ from: string; to?: string | null; condition?: string | null }>;
  join_nodes?: Array<{ id: string; wait_for: string[] }>;
}

export function buildGraphJson(options: {
  entryNode: string;
  maxIterations: number;
  nodes: GraphNodeSpec[];
  edges: GraphEdgeSpec[];
  joinNodes?: Array<{ id: string; waitFor: string[] }>;
}): Record<string, unknown> {
  const payload: GraphPayload = {
    entry_node: options.entryNode,
    max_iterations: options.maxIterations,
    nodes: options.nodes.map((n) => ({
      id: n.id,
      step_ref: n.stepRef,
      ...(n.inputs?.length ? { inputs: n.inputs } : {}),
      ...(n.outputs?.length ? { outputs: n.outputs } : {}),
    })),
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
  return payload as unknown as Record<string, unknown>;
}
