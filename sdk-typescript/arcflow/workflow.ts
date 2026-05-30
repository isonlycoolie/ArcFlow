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
