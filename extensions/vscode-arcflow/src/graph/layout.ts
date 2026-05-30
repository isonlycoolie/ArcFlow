import type {
  GraphDefinition,
  GraphLayout,
  GraphLayoutEdge,
  GraphLayoutNode,
  JoinNode,
  WorkflowDefinition,
} from "../types/rcs";

const NODE_WIDTH = 140;
const NODE_HEIGHT = 48;
const H_GAP = 80;
const V_GAP = 60;

function buildLinearLayout(workflow: WorkflowDefinition): GraphLayout {
  const sorted = [...workflow.steps].sort((a, b) => a.order - b.order);
  const nodes: GraphLayoutNode[] = sorted.map((step, index) => ({
    id: step.id,
    label: `Step ${step.order}`,
    stepRef: step.id,
    x: 40 + index * (NODE_WIDTH + H_GAP),
    y: 80,
    isEntry: index === 0,
    isJoin: false,
  }));

  const edges: GraphLayoutEdge[] = [];
  for (let i = 0; i < nodes.length - 1; i++) {
    edges.push({ from: nodes[i].id, to: nodes[i + 1].id });
  }

  return {
    nodes,
    edges,
    mode: "linear",
    workflowName: workflow.name,
    warnings: [],
  };
}

function layerNodes(graph: GraphDefinition, joinNodes: JoinNode[]): Map<string, number> {
  const layers = new Map<string, number>();
  const adjacency = new Map<string, string[]>();

  for (const node of graph.nodes) {
    adjacency.set(node.id, []);
  }
  for (const edge of graph.edges) {
    if (edge.to) {
      adjacency.get(edge.from)?.push(edge.to);
    }
  }

  const queue: string[] = [graph.entry_node];
  layers.set(graph.entry_node, 0);

  while (queue.length > 0) {
    const current = queue.shift()!;
    const layer = layers.get(current) ?? 0;
    for (const next of adjacency.get(current) ?? []) {
      const nextLayer = layer + 1;
      if (!layers.has(next) || (layers.get(next) ?? 0) < nextLayer) {
        layers.set(next, nextLayer);
        queue.push(next);
      }
    }
  }

  for (const join of joinNodes) {
    if (!layers.has(join.id)) {
      const maxWait = Math.max(
        0,
        ...join.wait_for.map((id) => (layers.has(id) ? (layers.get(id) ?? 0) + 1 : 1)),
      );
      layers.set(join.id, maxWait);
    }
  }

  return layers;
}

function buildGraphLayout(
  workflow: WorkflowDefinition,
  graph: GraphDefinition,
): GraphLayout {
  const joinNodes = graph.join_nodes ?? [];
  const joinIds = new Set(joinNodes.map((j) => j.id));
  const layers = layerNodes(graph, joinNodes);

  const byLayer = new Map<number, string[]>();
  for (const node of graph.nodes) {
    const layer = layers.get(node.id) ?? 0;
    if (!byLayer.has(layer)) {
      byLayer.set(layer, []);
    }
