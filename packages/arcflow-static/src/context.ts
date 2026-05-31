import { StaticConfigurationError } from "./errors.js";

export type PriorStepsMode = "all" | "last" | "none";

export class ContextPolicy {
  readonly includePriorSteps: PriorStepsMode;
  readonly includeRunInput: boolean;
  readonly maxPriorStepChars: number;

  constructor(options: {
    includePriorSteps?: PriorStepsMode;
    includeRunInput?: boolean;
    maxPriorStepChars?: number;
  } = {}) {
    this.includePriorSteps = options.includePriorSteps ?? "all";
    if (!["all", "last", "none"].includes(this.includePriorSteps)) {
      throw new StaticConfigurationError(
        "[ArcFlow] includePriorSteps must be 'all', 'last', or 'none'.",
      );
    }
    this.includeRunInput = options.includeRunInput ?? true;
    this.maxPriorStepChars = options.maxPriorStepChars ?? 4096;
    if (this.maxPriorStepChars < 256) {
      throw new StaticConfigurationError("[ArcFlow] maxPriorStepChars must be at least 256.");
    }
  }

  toJson(): Record<string, unknown> {
    return {
      include_prior_steps: this.includePriorSteps,
      include_run_input: this.includeRunInput,
      max_prior_step_chars: this.maxPriorStepChars,
    };
  }
}

export class ToolExecutionConfig {
  readonly mode: "legacy_eager" | "llm_select";
  readonly maxIterations: number;

  constructor(options: { mode?: "legacy_eager" | "llm_select"; maxIterations?: number } = {}) {
    this.mode = options.mode ?? "llm_select";
    if (this.mode !== "legacy_eager" && this.mode !== "llm_select") {
      throw new StaticConfigurationError(
        "[ArcFlow] tool_execution.mode must be 'legacy_eager' or 'llm_select'.",
      );
    }
    this.maxIterations = options.maxIterations ?? 5;
    if (this.maxIterations < 1 || this.maxIterations > 20) {
      throw new StaticConfigurationError(
        "[ArcFlow] maxIterations must be between 1 and 20.",
      );
    }
  }

  toJson(): Record<string, unknown> {
    return { mode: this.mode, max_iterations: this.maxIterations };
  }
}
