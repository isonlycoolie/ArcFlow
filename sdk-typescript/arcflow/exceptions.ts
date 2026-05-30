export class ArcFlowError extends Error {
  constructor(message: string) {
    super(message);
    this.name = new.target.name;
  }
}

export class WorkflowConfigurationError extends ArcFlowError {}

export class WorkflowExecutionError extends ArcFlowError {
  readonly runId?: string;
  readonly failedStep?: string;

  constructor(message: string, runId?: string, failedStep?: string) {
    super(message);
    this.runId = runId;
    this.failedStep = failedStep;
  }
}

export class ProviderConfigurationError extends ArcFlowError {}

export class ProviderExecutionError extends ArcFlowError {
  readonly providerId?: string;
  readonly runId?: string;
  readonly failedStep?: string;

  constructor(
    message: string,
    providerId?: string,
    runId?: string,
    failedStep?: string,
  ) {
    super(message);
    this.providerId = providerId;
    this.runId = runId;
    this.failedStep = failedStep;
  }
}

export class TraceNotFoundError extends ArcFlowError {}

export class RetryExhaustedError extends ArcFlowError {
  readonly attemptsMade?: number;

  constructor(message: string, attemptsMade?: number) {
    super(message);
    this.attemptsMade = attemptsMade;
  }
}

export class WorkflowTimeoutError extends ArcFlowError {
  readonly timeoutType?: string;

  constructor(message: string, timeoutType?: string) {
    super(message);
    this.timeoutType = timeoutType;
  }
}

export function mapNativeError(err: unknown): ArcFlowError {
  const message = err instanceof Error ? err.message : String(err);
  if (message.includes("ProviderExecutionError|")) {
    const [, providerId, runId, failedStep, ...rest] = message.split("|");
    return new ProviderExecutionError(
      rest.join("|") || message,
      providerId,
      runId,
      failedStep,
    );
  }
  if (message.includes("WorkflowExecutionError|")) {
    const [, runId, failedStep, ...rest] = message.split("|");
    return new WorkflowExecutionError(rest.join("|") || message, runId, failedStep);
  }
  if (message.includes("failed after") && message.includes("attempts")) {
    const match = message.match(/after (\d+) attempts/);
    return new RetryExhaustedError(message, match ? Number(match[1]) : undefined);
  }
  if (message.toLowerCase().includes("timed out")) {
    const timeoutType = message.includes("Workflow") ? "workflow" : "step";
    return new WorkflowTimeoutError(message, timeoutType);
  }
  if (message.includes("No trace found")) {
    return new TraceNotFoundError(message);
  }
  if (
    message.includes("invalid") ||
    message.includes("must be") ||
    message.includes("Cannot run")
  ) {
    return new WorkflowConfigurationError(message);
  }
  return new ArcFlowError(message);
}
