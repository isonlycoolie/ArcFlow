"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NODE_HEIGHT = exports.NODE_WIDTH = void 0;
exports.computeGraphLayout = computeGraphLayout;
exports.layoutDimensions = layoutDimensions;
const NODE_WIDTH = 140;
exports.NODE_WIDTH = NODE_WIDTH;
const NODE_HEIGHT = 48;
exports.NODE_HEIGHT = NODE_HEIGHT;
const H_GAP = 80;
const V_GAP = 60;
function buildLinearLayout(workflow) {
    const sorted = [...workflow.steps].sort((a, b) => a.order - b.order);
    const nodes = sorted.map((step, index) => ({
        id: step.id,
        label: `Step ${step.order}`,
        stepRef: step.id,
        x: 40 + index * (NODE_WIDTH + H_GAP),
        y: 80,
        isEntry: index === 0,
        isJoin: false,
    }));
    const edges = [];
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
function layerNodes(graph, joinNodes) {
    const layers = new Map();
    const adjacency = new Map();
    for (const node of graph.nodes) {
        adjacency.set(node.id, []);
    }
    for (const edge of graph.edges) {
        if (edge.to) {
            adjacency.get(edge.from)?.push(edge.to);
        }
    }
    const queue = [graph.entry_node];
    layers.set(graph.entry_node, 0);
    while (queue.length > 0) {
        const current = queue.shift();
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
            const maxWait = Math.max(0, ...join.wait_for.map((id) => (layers.has(id) ? (layers.get(id) ?? 0) + 1 : 1)));
            layers.set(join.id, maxWait);
        }
    }
    return layers;
}
function buildGraphLayout(workflow, graph) {
    const joinNodes = graph.join_nodes ?? [];
    const joinIds = new Set(joinNodes.map((j) => j.id));
    const layers = layerNodes(graph, joinNodes);
    const byLayer = new Map();
    for (const node of graph.nodes) {
        const layer = layers.get(node.id) ?? 0;
        if (!byLayer.has(layer)) {
            byLayer.set(layer, []);
        }
        byLayer.get(layer).push(node.id);
    }
    for (const join of joinNodes) {
        const layer = layers.get(join.id) ?? 0;
        if (!byLayer.has(layer)) {
            byLayer.set(layer, []);
        }
        if (!byLayer.get(layer).includes(join.id)) {
            byLayer.get(layer).push(join.id);
        }
    }
    const nodes = [];
    for (const [layer, ids] of byLayer.entries()) {
        ids.forEach((id, indexInLayer) => {
            const graphNode = graph.nodes.find((n) => n.id === id);
            const isJoin = joinIds.has(id);
            nodes.push({
                id,
                label: isJoin ? `join: ${id}` : id,
