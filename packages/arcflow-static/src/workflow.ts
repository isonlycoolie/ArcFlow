import { Agent } from "./agent.js";
import { buildExecConfig, type RetryOptions } from "./exec-config.js";
import { StaticConfigurationError } from "./errors.js";
import type { ExternalBinding } from "./external.js";
import { buildGraphJson } from "./graph.js";
import { HitlConfig } from "./hitl.js";
import { newId } from "./ids.js";

export interface WorkflowConfig {
  name?: string;
  runtime?: string;
  graph?: boolean;
}

interface GraphNodeRecord {
  nodeId: string;
  agent: Agent;
  stepId: string;
  inputs?: string[];
  outputs?: string[];
}

export class Workflow {
  readonly name: string;
  readonly runtimeUrl: string | undefined;
  private readonly graphMode: boolean;
  private readonly steps: Array<{ agent: Agent; hitl?: HitlConfig; stepId: string }> = [];
  private readonly graphNodes = new Map<string, GraphNodeRecord>();
  private readonly graphEdges: Array<{ from: string; to?: string | null; condition?: string | null }> = [];
  private readonly graphJoins: Array<{ id: string; waitFor: string[] }> = [];
  private externalBindings: ExternalBinding[] = [];
  private entryNode: string | null = null;
  private maxIterations = 100;
  private workflowId: string | null = null;
  private retryOptions: RetryOptions | undefined;
  private workflowTimeoutSeconds: number | undefined;
  private stepTimeoutSeconds: number | undefined;
  private recoveryEnabled = false;
  registryRef: { name: string; version: string } | undefined;

  constructor(config: WorkflowConfig = {}) {
    const trimmed = (config.name ?? "default").trim();
    if (!trimmed) {
      throw new StaticConfigurationError("[ArcFlow] Workflow name must be non-empty.");
    }
    this.name = trimmed;
    this.graphMode = config.graph === true;
    this.runtimeUrl = config.runtime?.trim().replace(/\/$/, "") || undefined;
  }

  step(agent: Agent, options: { hitl?: HitlConfig } = {}): this {
    if (this.graphMode) {
      throw new StaticConfigurationError("[ArcFlow] Graph workflows use node(), not step().");
    }
    this.steps.push({ agent, hitl: options.hitl, stepId: newId() });
    return this;
  }

  node(nodeId: string, agent: Agent, options: { inputs?: string[]; outputs?: string[] } = {}): this {
    if (!this.graphMode) {
      throw new StaticConfigurationError("[ArcFlow] node() requires Workflow({ graph: true }).");
    }
    const trimmed = nodeId.trim();
    if (!trimmed) {
      throw new StaticConfigurationError("[ArcFlow] Graph node id must be non-empty.");
    }
    if (this.graphNodes.has(trimmed)) {
      throw new StaticConfigurationError(`[ArcFlow] Duplicate graph node id '${trimmed}'.`);
    }
    this.graphNodes.set(trimmed, {
      nodeId: trimmed,
      agent,
      stepId: newId(),
      inputs: options.inputs,
      outputs: options.outputs,
    });
    if (!this.entryNode) this.entryNode = trimmed;
    return this;
  }

  addEdge(fromId: string, toId?: string | null, options: { condition?: string | null } = {}): this {
    if (!this.graphMode) {
      throw new StaticConfigurationError("[ArcFlow] addEdge() requires graph mode.");
    }
    this.graphEdges.push({ from: fromId.trim(), to: toId ?? null, condition: options.condition ?? null });
    return this;
  }

  joinNode(joinId: string, waitFor: string[]): this {
    if (!this.graphMode) {
      throw new StaticConfigurationError("[ArcFlow] joinNode() requires graph mode.");
    }
    if (!this.graphNodes.has(joinId.trim())) {
      throw new StaticConfigurationError(`[ArcFlow] Join node '${joinId}' is not registered.`);
    }
    this.graphJoins.push({ id: joinId.trim(), waitFor: waitFor.map((id) => id.trim()) });
    return this;
  }

  setEntry(nodeId: string): this {
    const trimmed = nodeId.trim();
    if (!this.graphNodes.has(trimmed)) {
      throw new StaticConfigurationError(`[ArcFlow] Entry node '${trimmed}' is not registered.`);
    }
    this.entryNode = trimmed;
    return this;
  }

  withExternalBindings(bindings: ExternalBinding[]): this {
    this.externalBindings = bindings;
    return this;
  }

  withRetry(maxAttempts: number, options: Omit<RetryOptions, "maxAttempts"> = {}): this {
    this.retryOptions = { maxAttempts, ...options };
    return this;
  }

  withTimeout(seconds: number): this {
    this.workflowTimeoutSeconds = seconds;
    return this;
  }

  enableRecovery(): this {
    this.recoveryEnabled = true;
    return this;
  }

  buildRunPayload(input: string, initialState?: Record<string, unknown>): Record<string, unknown> {
    const execConfig = buildExecConfig({
      retry: this.retryOptions,
      workflowTimeoutSeconds: this.workflowTimeoutSeconds,
      stepTimeoutSeconds: this.stepTimeoutSeconds,
      recoveryEnabled: this.recoveryEnabled,
      initialState,
    });
    if (this.registryRef) {
      const payload: Record<string, unknown> = {
        workflow_ref: { name: this.registryRef.name, version: this.registryRef.version },
        input,
      };
      if (execConfig) payload.exec_config = execConfig;
      return payload;
    }
    const { agents, steps, workflowBody } = this.compileWorkflow();
    const payload: Record<string, unknown> = {
      workflow: workflowBody,
      agents: agents.map((a) => a.bindingRow()),
      input,
    };
    if (execConfig) payload.exec_config = execConfig;
    return payload;
  }

  buildPublishPayload(publishedBy?: string): Record<string, unknown> {
    const { agents, workflowBody } = this.compileWorkflow();
    const payload: Record<string, unknown> = {
      workflow: workflowBody,
      agents: agents.map((a) => a.bindingRow()),
    };
    if (publishedBy) payload.published_by = publishedBy;
    return payload;
  }

  private compileWorkflow(): {
    agents: Agent[];
    steps: Array<{ stepId: string; agentId: string; order: number; hitl?: HitlConfig }>;
    workflowBody: Record<string, unknown>;
  } {
    const compiled = this.agentsAndSteps();
    const workflowId = this.workflowId ?? newId();
    this.workflowId = workflowId;
    const workflowBody: Record<string, unknown> = {
      id: workflowId,
      name: this.name,
      steps: compiled.steps.map((s) => {
        const row: Record<string, unknown> = {
          id: s.stepId,
          agent_id: s.agentId,
          order: s.order,
        };
        if (s.hitl) row.hitl = s.hitl.toJson();
        return row;
      }),
      execution_mode: this.graphMode ? "graph" : "linear",
    };
    if (this.graphMode) {
      if (!this.entryNode) {
        throw new StaticConfigurationError("[ArcFlow] Graph workflow has no entry node.");
      }
      workflowBody.graph = buildGraphJson({
        entryNode: this.entryNode,
        maxIterations: this.maxIterations,
        nodes: [...this.graphNodes.values()].map((n) => ({
          id: n.nodeId,
          stepRef: n.stepId,
          inputs: n.inputs,
          outputs: n.outputs,
        })),
        edges: this.graphEdges,
        joinNodes: this.graphJoins,
      });
    }
    if (this.externalBindings.length) {
      workflowBody.external_bindings = this.externalBindings;
    }
    return { agents: compiled.agents, steps: compiled.steps, workflowBody };
  }

  private agentsAndSteps(): {
    agents: Agent[];
    steps: Array<{ stepId: string; agentId: string; order: number; hitl?: HitlConfig }>;
  } {
    if (this.graphMode) {
      const agents: Agent[] = [];
      const steps: Array<{ stepId: string; agentId: string; order: number; hitl?: HitlConfig }> = [];
      let order = 1;
      for (const record of this.graphNodes.values()) {
        agents.push(record.agent);
        steps.push({ stepId: record.stepId, agentId: record.agent.agentId, order: order++ });
      }
      return { agents, steps };
    }
    return {
      agents: this.steps.map((row) => row.agent),
      steps: this.steps.map((row, index) => ({
        stepId: row.stepId,
        agentId: row.agent.agentId,
        order: index + 1,
        hitl: row.hitl,
      })),
    };
  }
}

export function resolveWorkflow(name: string, version: string, runtime: string): Workflow {
  const wf = new Workflow({ name, runtime });
  wf.registryRef = { name, version };
  return wf;
}
