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
