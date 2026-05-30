import * as fs from "fs";
import * as path from "path";
import type { GraphLayout, GraphLayoutNode } from "../types/rcs";

export interface LayoutSidecar {
  workflowName: string;
  positions: Record<string, { x: number; y: number }>;
}

export function sidecarPath(workflowFile: string): string {
  return workflowFile.replace(/\.arcflow\.json$/i, ".arcflow.layout.json");
}

export function loadLayoutSidecar(workflowFile: string): LayoutSidecar | undefined {
  const file = sidecarPath(workflowFile);
  if (!fs.existsSync(file)) {
    return undefined;
  }
  try {
    return JSON.parse(fs.readFileSync(file, "utf8")) as LayoutSidecar;
  } catch {
    return undefined;
  }
}

export function saveLayoutSidecar(workflowFile: string, sidecar: LayoutSidecar): void {
  const file = sidecarPath(workflowFile);
  fs.writeFileSync(file, JSON.stringify(sidecar, null, 2), "utf8");
}

export function mergeSidecarPositions(
  layout: GraphLayout,
  sidecar: LayoutSidecar | undefined,
): GraphLayout {
  if (!sidecar?.positions) {
    return layout;
  }
  const nodes: GraphLayoutNode[] = layout.nodes.map((node) => {
    const saved = sidecar.positions[node.id];
    if (!saved) {
      return node;
    }
    return { ...node, x: saved.x, y: saved.y };
  });
  return { ...layout, nodes };
}

export function layoutToSidecar(layout: GraphLayout): LayoutSidecar {
  const positions: Record<string, { x: number; y: number }> = {};
  for (const node of layout.nodes) {
    positions[node.id] = { x: node.x, y: node.y };
  }
  return { workflowName: layout.workflowName, positions };
}

export function resolveWorkflowPath(document: { uri: { fsPath: string } }): string {
  return path.normalize(document.uri.fsPath);
}
