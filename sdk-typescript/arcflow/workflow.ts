import { randomUUID } from "node:crypto";

import { Agent } from "./agent.js";
import {
  mapNativeError,
  TraceNotFoundError,
  WorkflowConfigurationError,
} from "./exceptions.js";
import type { Provider } from "./provider.js";
import { toWorkflowResult, type WorkflowResult } from "./result.js";
import { traceFromJson, type TraceResult } from "./trace.js";

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
}

export interface RunOptions {
  provider?: Provider;
}

export class Workflow {
  private readonly name: string;
  private readonly steps: Agent[] = [];
  private lastRunId: string | null = null;

  constructor(config: WorkflowConfig = {}) {
    const trimmed = (config.name ?? "default").trim();
    if (!trimmed) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Workflow name must be a non-empty string.",
      );
    }
    this.name = trimmed;
  }

  step(agent: Agent): this {
    if (!(agent instanceof Agent)) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] workflow.step() requires an Agent instance.",
      );
    }
    this.steps.push(agent);
    return this;
  }

  async run(input: string, options: RunOptions = {}): Promise<WorkflowResult> {
    const trimmed = input.trim();
    if (!trimmed) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Workflow input must be a non-empty string.",
      );
    }
    if (this.steps.length === 0) {
      throw new WorkflowConfigurationError(
        "[ArcFlow] Cannot run a workflow with no steps.",
      );
    }
    const native = loadNative();
    const agents = this.steps.map((agent) => agent.bindingRow());
    const steps = this.steps.map((agent, index) => ({
      stepId: randomUUID(),
      agentId: agent.agentId,
      order: index + 1,
    }));
    const provider = options.provider?.bindingRow();
    try {
      const result = await native.executeWorkflow(
        this.name,
        randomUUID(),
        agents,
        steps,
        trimmed,
        provider,
      );
      this.lastRunId = result.runId;
      return toWorkflowResult(result);
    } catch (err) {
      throw mapNativeError(err);
    }
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
