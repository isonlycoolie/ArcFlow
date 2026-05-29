/** Human-in-the-loop configuration and errors. */

import { ArcFlowError } from "./exceptions.js";

export interface HitlConfigOptions {
  approvalKey: string;
  timeoutSeconds?: number;
  interrupt?: boolean;
}

export class HitlConfig {
  readonly approvalKey: string;
  readonly timeoutSeconds: number;
  readonly interrupt: boolean;

  constructor(options: HitlConfigOptions | string) {
    if (typeof options === "string") {
      this.approvalKey = options;
      this.timeoutSeconds = 3600;
      this.interrupt = true;
      return;
    }
    const key = options.approvalKey.trim();
    if (!key) {
      throw new Error("[ArcFlow] HitlConfig requires a non-empty approvalKey.");
    }
    this.approvalKey = key;
    this.timeoutSeconds = options.timeoutSeconds ?? 3600;
    this.interrupt = options.interrupt ?? true;
  }

  toJson(): string {
    return JSON.stringify({
      approval_key: this.approvalKey,
      timeout_seconds: this.timeoutSeconds,
      interrupt: this.interrupt,
    });
  }
}

export class HumanRejectedError extends ArcFlowError {
  readonly approvalKey: string | undefined;

  constructor(message: string, approvalKey?: string) {
    super(message);
    this.name = "HumanRejectedError";
    this.approvalKey = approvalKey;
  }
}

export class WorkflowInterruptedError extends ArcFlowError {
  readonly runId: string;
  readonly approvalKey: string;
  readonly expiresAt: string | undefined;

  constructor(
    message: string,
    runId: string,
    approvalKey: string,
    expiresAt?: string,
  ) {
    super(message);
    this.name = "WorkflowInterruptedError";
    this.runId = runId;
    this.approvalKey = approvalKey;
    this.expiresAt = expiresAt;
  }
}
