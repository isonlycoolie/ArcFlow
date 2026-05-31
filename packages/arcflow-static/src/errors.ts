export class StaticConfigurationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "StaticConfigurationError";
  }
}

export class StaticExecutionError extends Error {
  readonly runId: string | undefined;
  readonly failedStep: string | undefined;

  constructor(message: string, runId?: string, failedStep?: string) {
    super(message);
    this.name = "StaticExecutionError";
    this.runId = runId;
    this.failedStep = failedStep;
  }
}

export class WorkflowInterruptedError extends Error {
  readonly runId: string;
  readonly approvalKey: string;
  readonly expiresAt: string | undefined;

  constructor(message: string, runId: string, approvalKey: string, expiresAt?: string) {
    super(message);
    this.name = "WorkflowInterruptedError";
    this.runId = runId;
    this.approvalKey = approvalKey;
    this.expiresAt = expiresAt;
  }
}
