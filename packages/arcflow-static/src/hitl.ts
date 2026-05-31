import { StaticConfigurationError } from "./errors.js";

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
      this.approvalKey = options.trim();
      this.timeoutSeconds = 3600;
      this.interrupt = true;
    } else {
      this.approvalKey = options.approvalKey.trim();
      this.timeoutSeconds = options.timeoutSeconds ?? 3600;
      this.interrupt = options.interrupt ?? true;
    }
    if (!this.approvalKey) {
      throw new StaticConfigurationError("[ArcFlow] HitlConfig requires a non-empty approvalKey.");
    }
  }

  toJson(): Record<string, unknown> {
    return {
      approval_key: this.approvalKey,
      timeout_seconds: this.timeoutSeconds,
      interrupt: this.interrupt,
    };
  }
}
