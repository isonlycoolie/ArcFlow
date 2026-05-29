import { randomUUID } from "node:crypto";

import { Agent } from "./agent.js";
import {
  mapNativeError,
  TraceNotFoundError,
  WorkflowConfigurationError,
  WorkflowExecutionError,
} from "./exceptions.js";
import { buildGraphJson } from "./graph.js";
import { runRemoteWorkflow } from "./remote.js";
import type { Provider } from "./provider.js";
import { toWorkflowResult, type WorkflowResult } from "./result.js";
import { traceFromJson, type TraceResult } from "./trace.js";
import { buildExecConfigJson, type RetryOptions } from "./types/fault.js";

type NativeBinding = {
  executeWorkflow: (
    workflowName: string,
    workflowId: string,
    agents: Array<{ id: string; name: string; role: string; instructions: string }>,
    steps: Array<{ stepId: string; agentId: string; order: number }>,
    runInput: string,
    provider?: {
      kind: string;
      model: string;
      maxTokens: number;
      temperature: number;
    },
    execConfigJson?: string,
    graphJson?: string,
  ) => Promise<{
    output: string;
    runId: string;
    stepCount: number;
    traceEventsJson: string;
  }>;
  executeResumeWorkflow: (
    workflowName: string,
    workflowId: string,
    agents: Array<{ id: string; name: string; role: string; instructions: string }>,
    steps: Array<{ stepId: string; agentId: string; order: number }>,
    originalRunId: string,
    provider?: {
      kind: string;
      model: string;
      maxTokens: number;
      temperature: number;
    },
    execConfigJson?: string,
  ) => Promise<{
    output: string;
    runId: string;
    stepCount: number;
    traceEventsJson: string;
  }>;
  getExecutionTraceJson: (runId: string) => string;
};

function loadNative(): NativeBinding {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  return require("../index.native.js") as NativeBinding;
}

export interface WorkflowConfig {
  name?: string;
  graph?: boolean;
  runtime?: string;
}

export interface RunOptions {
  provider?: Provider;
}

interface GraphNodeRecord {
  nodeId: string;
  agent: Agent;
  stepId: string;
}

export class Workflow {
  private readonly name: string;
  private readonly graphMode: boolean;
  readonly runtimeUrl: string | undefined;
  private readonly steps: Agent[] = [];
  private readonly graphNodes = new Map<string, GraphNodeRecord>();
  private readonly graphEdges: Array<{
    from: string;
    to?: string | null;
    condition?: string | null;
  }> = [];
  private entryNode: string | null = null;
  private maxIterations = 100;
  private workflowId: string | null = null;
  private lastRunId: string | null = null;
  private hasRun = false;
  private retryOptions: RetryOptions | undefined;
  private workflowTimeoutSeconds: number | undefined;
  private stepTimeoutSeconds: number | undefined;
  private recoveryEnabled = false;

  constructor(config: WorkflowConfig = {}) {
    const trimmed = (config.name ?? "default").trim();
    if (!trimmed) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Workflow name must be a non-empty string.",
      );
    }
    this.name = trimmed;
    this.graphMode = config.graph === true;
    this.runtimeUrl = config.runtime?.trim().replace(/\/$/, "") || undefined;
  }

  step(agent: Agent): this {
    if (this.graphMode) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Graph workflows use node() — step() is not allowed when graph=true.",
      );
    }
    if (!(agent instanceof Agent)) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] workflow.step() requires an Agent instance.",
      );
    }
    this.steps.push(agent);
    return this;
  }

  node(nodeId: string, agent: Agent): this {
    if (!this.graphMode) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] node() requires Workflow({ graph: true }).",
      );
    }
    if (this.hasRun) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] workflow.node() must be called before workflow.run().",
      );
    }
    const trimmed = nodeId.trim();
    if (!trimmed) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Graph node id must be a non-empty string.",
      );
    }
    if (!(agent instanceof Agent)) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] workflow.node() requires an Agent instance.",
      );
    }
    if (this.graphNodes.has(trimmed)) {
      throw new WorkflowConfigurationError(
        `[ArcFlow] Duplicate graph node id '${trimmed}'.`,
      );
    }
    this.graphNodes.set(trimmed, {
      nodeId: trimmed,
      agent,
      stepId: randomUUID(),
    });
    if (!this.entryNode) {
      this.entryNode = trimmed;
    }
    return this;
  }

  addEdge(
    fromId: string,
    toId?: string | null,
    options: { condition?: string | null } = {},
  ): this {
    if (!this.graphMode) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] addEdge() requires Workflow({ graph: true }).",
      );
    }
    this.graphEdges.push({
      from: fromId.trim(),
      to: toId ?? null,
      condition: options.condition ?? null,
    });
    return this;
  }

  setEntry(nodeId: string): this {
    const trimmed = nodeId.trim();
    if (!this.graphNodes.has(trimmed)) {
      throw new WorkflowConfigurationError(
        `[ArcFlow] Entry node '${trimmed}' is not registered.`,
      );
    }
    this.entryNode = trimmed;
    return this;
  }

  withMaxIterations(count: number): this {
    if (count < 1) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] max_iterations must be at least 1.",
      );
    }
    this.maxIterations = count;
    return this;
  }

  withRetry(maxAttempts: number, options: Omit<RetryOptions, "maxAttempts"> = {}): this {
    if (this.hasRun) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] withRetry() must be called before workflow.run().",
      );
    }
    if (maxAttempts < 1) {
      throw new WorkflowConfigurationError(
        `[ArcFlow] retry maxAttempts must be at least 1. Got ${maxAttempts}.`,
      );
    }
    this.retryOptions = { maxAttempts, ...options };
    return this;
  }

  withTimeout(seconds: number): this {
    if (seconds <= 0) {
      throw new WorkflowConfigurationError(
        `[ArcFlow] Workflow timeout must be positive. Got ${seconds}s.`,
      );
    }
    this.workflowTimeoutSeconds = seconds;
    return this;
  }

  withStepTimeout(seconds: number): this {
    if (seconds <= 0) {
      throw new WorkflowConfigurationError(
        `[ArcFlow] Step timeout must be positive. Got ${seconds}s.`,
      );
    }
    this.stepTimeoutSeconds = seconds;
    return this;
  }

  enableRecovery(): this {
    this.recoveryEnabled = true;
    return this;
  }

  private execConfigJson(): string | undefined {
    return buildExecConfigJson({
      retry: this.retryOptions,
      workflowTimeoutSeconds: this.workflowTimeoutSeconds,
      stepTimeoutSeconds: this.stepTimeoutSeconds,
      recoveryEnabled: this.recoveryEnabled,
    });
  }

  private agentsAndSteps(): {
    agents: Agent[];
    steps: Array<{ stepId: string; agentId: string; order: number }>;
  } {
    if (this.graphMode) {
      const agents: Agent[] = [];
      const steps: Array<{ stepId: string; agentId: string; order: number }> = [];
      let order = 1;
      for (const record of this.graphNodes.values()) {
        agents.push(record.agent);
        steps.push({
          stepId: record.stepId,
          agentId: record.agent.agentId,
          order: order++,
        });
      }
      return { agents, steps };
    }
    return {
      agents: this.steps,
      steps: this.steps.map((agent, index) => ({
        stepId: randomUUID(),
        agentId: agent.agentId,
        order: index + 1,
      })),
    };
  }

  private graphJson(): string | undefined {
    if (!this.graphMode) {
      return undefined;
    }
