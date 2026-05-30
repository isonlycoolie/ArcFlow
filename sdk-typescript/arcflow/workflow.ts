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
import { HitlConfig } from "./hitl.js";
import type { StreamEvent, StreamRunResult } from "./stream.js";

type NativeBinding = {
  executeWorkflow: (
    workflowName: string,
    workflowId: string,
    agents: Array<{ id: string; name: string; role: string; instructions: string }>,
    steps: Array<{ stepId: string; agentId: string; order: number; hitlJson?: string }>,
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
    steps: Array<{ stepId: string; agentId: string; order: number; hitlJson?: string }>,
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
  executeWorkflowStream: (
    workflowName: string,
    workflowId: string,
    agents: Array<{ id: string; name: string; role: string; instructions: string }>,
    steps: Array<{ stepId: string; agentId: string; order: number; hitlJson?: string }>,
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
    eventsJson: string;
    output: string;
    runId: string;
    stepCount: number;
    traceEventsJson: string;
  }>;
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
  private readonly steps: Array<{ agent: Agent; hitl?: HitlConfig }> = [];
  private readonly graphNodes = new Map<string, GraphNodeRecord>();
  private readonly graphEdges: Array<{
    from: string;
    to?: string | null;
    condition?: string | null;
  }> = [];
  private readonly graphJoins: Array<{ id: string; waitFor: string[] }> = [];
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

  step(agent: Agent, options: { hitl?: HitlConfig } = {}): this {
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
    this.steps.push({ agent, hitl: options.hitl });
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

  joinNode(joinId: string, waitFor: string[]): this {
    if (!this.graphMode) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] joinNode() requires Workflow({ graph: true }).",
      );
    }
    const trimmed = joinId.trim();
    if (!this.graphNodes.has(trimmed)) {
      throw new WorkflowConfigurationError(
        `[ArcFlow] Join node '${trimmed}' is not registered.`,
      );
    }
    if (!waitFor.length) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] joinNode waitFor must list at least one branch node.",
      );
    }
    this.graphJoins.push({
      id: trimmed,
      waitFor: waitFor.map((id) => id.trim()),
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
    steps: Array<{ stepId: string; agentId: string; order: number; hitlJson?: string }>;
  } {
    if (this.graphMode) {
      const agents: Agent[] = [];
      const steps: Array<{ stepId: string; agentId: string; order: number; hitlJson?: string }> = [];
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
      agents: this.steps.map((row) => row.agent),
      steps: this.steps.map((row, index) => ({
        stepId: randomUUID(),
        agentId: row.agent.agentId,
        order: index + 1,
        hitlJson: row.hitl?.toJson(),
      })),
    };
  }

  private graphJson(): string | undefined {
    if (!this.graphMode) {
      return undefined;
    }
    if (!this.entryNode) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Graph workflow has no entry node.",
      );
    }
    return buildGraphJson({
      entryNode: this.entryNode,
      maxIterations: this.maxIterations,
      nodes: [...this.graphNodes.values()].map((n) => ({
        id: n.nodeId,
        stepId: n.stepId,
      })),
      edges: this.graphEdges,
      joinNodes: this.graphJoins,
    });
  }

  async resume(runId: string): Promise<WorkflowResult> {
    if (!this.recoveryEnabled) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] workflow.resume() requires enableRecovery().",
      );
    }
    if (!runId.trim()) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] resume() requires a non-empty run_id.",
      );
    }
    if (!this.workflowId) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Cannot resume — no prior run on this workflow instance.",
      );
    }
    const native = loadNative();
    const { agents, steps } = this.agentsAndSteps();
    try {
      const result = await native.executeResumeWorkflow(
        this.name,
        this.workflowId,
        agents.map((agent) => agent.bindingRow()),
        steps,
        runId.trim(),
        undefined,
        this.execConfigJson(),
      );
      this.lastRunId = result.runId;
      return toWorkflowResult(result);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  buildRunPayload(input: string, execConfigJson?: string): Record<string, unknown> {
    const { agents, steps } = this.agentsAndSteps();
    const workflowId = this.workflowId ?? randomUUID();
    this.workflowId = workflowId;
    const workflowBody: Record<string, unknown> = {
      id: workflowId,
      name: this.name,
      steps: steps.map((s) => {
        const row: Record<string, unknown> = {
          id: s.stepId,
          agent_id: s.agentId,
          order: s.order,
        };
        if (s.hitlJson) {
          row.hitl = JSON.parse(s.hitlJson);
        }
        return row;
      }),
      execution_mode: this.graphMode ? "graph" : "linear",
    };
    if (this.graphMode) {
      workflowBody.graph = JSON.parse(this.graphJson()!);
    }
    const payload: Record<string, unknown> = {
      workflow: workflowBody,
      agents: agents.map((agent) => ({
        id: agent.agentId,
        name: agent.name,
        role: agent.role,
        instructions: agent.instructions,
      })),
      input,
    };
    if (execConfigJson) {
      payload.exec_config = JSON.parse(execConfigJson);
    }
    return payload;
  }

  async run(input: string, options: RunOptions = {}): Promise<WorkflowResult> {
    const trimmed = input.trim();
    if (!trimmed) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Workflow input must be a non-empty string.",
      );
    }
    if (this.graphMode) {
      if (this.graphNodes.size === 0) {
        throw new WorkflowConfigurationError(
          "[ArcFlow] Cannot run a graph workflow with no nodes.",
        );
      }
    } else if (this.steps.length === 0) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Cannot run a workflow with no steps.",
      );
    }
    if (!this.workflowId) {
      this.workflowId = randomUUID();
    }
    if (this.runtimeUrl) {
      const result = await runRemoteWorkflow(
        this,
        trimmed,
        this.execConfigJson(),
      );
      this.lastRunId = result.runId;
      this.hasRun = true;
      return result;
    }
    const native = loadNative();
    const { agents, steps } = this.agentsAndSteps();
    const provider = options.provider?.bindingRow();
    try {
      const result = await native.executeWorkflow(
        this.name,
        this.workflowId,
        agents.map((agent) => agent.bindingRow()),
        steps,
        trimmed,
        provider,
        this.execConfigJson(),
        this.graphJson(),
      );
      this.lastRunId = result.runId;
      this.hasRun = true;
      return toWorkflowResult(result);
    } catch (err) {
      if (err instanceof Error && err.message.includes("WorkflowExecutionError|")) {
        const [, runId] = err.message.split("|");
        if (runId) {
          this.lastRunId = runId;
          this.hasRun = true;
        }
        throw mapNativeError(err);
      }
      throw mapNativeError(err);
    }
  }

  async *runStream(
    input: string,
    options: RunOptions = {},
  ): AsyncGenerator<StreamEvent, StreamRunResult, undefined> {
    const trimmed = input.trim();
    if (!trimmed) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Workflow input must be a non-empty string.",
      );
    }
    if (this.graphMode) {
      if (this.graphNodes.size === 0) {
        throw new WorkflowConfigurationError(
          "[ArcFlow] Cannot run a graph workflow with no nodes.",
        );
      }
    } else if (this.steps.length === 0) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Cannot run a workflow with no steps.",
      );
    }
    if (!this.workflowId) {
      this.workflowId = randomUUID();
    }
    const native = loadNative();
    const { agents, steps } = this.agentsAndSteps();
    const provider = options.provider?.bindingRow();
    const execConfigJson = buildExecConfigJson({
      retry: this.retryOptions,
      workflowTimeoutSeconds: this.workflowTimeoutSeconds,
      stepTimeoutSeconds: this.stepTimeoutSeconds,
      recoveryEnabled: this.recoveryEnabled,
      stream: true,
    });
    try {
      const result = await native.executeWorkflowStream(
        this.name,
        this.workflowId,
        agents.map((agent) => agent.bindingRow()),
        steps,
        trimmed,
        provider,
        execConfigJson,
        this.graphJson(),
      );
      const events = JSON.parse(result.eventsJson) as StreamEvent[];
      for (const event of events) {
        yield event;
      }
      this.lastRunId = result.runId;
      this.hasRun = true;
      return {
        output: result.output,
        runId: result.runId,
        stepCount: result.stepCount,
        traceEventsJson: result.traceEventsJson,
      };
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  async test(
    cases: Array<{
      name?: string;
      input?: string;
      expected_output?: string;
      stub_responses?: Record<string, unknown>;
    }>,
  ): Promise<Array<{ name: string; passed: boolean; output: string }>> {
    const { agents, steps } = this.agentsAndSteps();
    const native = loadNative();
    const results: Array<{ name: string; passed: boolean; output: string }> = [];
    for (const testCase of cases) {
      const name = String(testCase.name ?? "case");
      const runInput = String(testCase.input ?? "");
      let stubResponses = testCase.stub_responses;
      if (stubResponses === undefined && testCase.expected_output !== undefined) {
        stubResponses = { step_1: { output: testCase.expected_output } };
      }
      const execConfigJson = buildExecConfigJson({
        retry: this.retryOptions,
        workflowTimeoutSeconds: this.workflowTimeoutSeconds,
        stepTimeoutSeconds: this.stepTimeoutSeconds,
        recoveryEnabled: false,
        test: { stub_responses: stubResponses ?? {} },
      });
      const workflowId = randomUUID();
      try {
        const result = await native.executeWorkflow(
          this.name,
          workflowId,
          agents.map((agent) => agent.bindingRow()),
          steps,
          runInput,
          undefined,
          execConfigJson,
          this.graphJson(),
        );
        const expected = testCase.expected_output;
        const passed = expected === undefined || result.output === expected;
        results.push({ name, passed, output: result.output });
      } catch (err) {
        throw mapNativeError(err);
      }
    }
    return results;
  }

  trace(): TraceResult {
    if (!this.lastRunId) {
      throw new TraceNotFoundError(
        "[ArcFlow] No workflow run yet. Call workflow.run() before trace().",
      );
    }
    const native = loadNative();
    try {
      return traceFromJson(native.getExecutionTraceJson(this.lastRunId));
    } catch (err) {
      throw mapNativeError(err);
    }
  }
}

export { WorkflowExecutionError };
