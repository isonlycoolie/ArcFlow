import {
  PROVIDER_DEFAULT_MAX_TOKENS,
  PROVIDER_DEFAULT_TEMPERATURE,
} from "./constants.js";
import { ProviderConfigurationError } from "./exceptions.js";

export interface ProviderConfig {
  model: string;
  maxTokens?: number;
  temperature?: number;
}

function validateProvider(kind: string, config: ProviderConfig) {
  if (!config.model.trim()) {
    throw new ProviderConfigurationError(
      `[ArcFlow] ${kind} model must be a non-empty string.`,
    );
  }
  const temperature = config.temperature ?? PROVIDER_DEFAULT_TEMPERATURE;
  if (temperature < 0 || temperature > 1) {
    throw new ProviderConfigurationError(
      `[ArcFlow] ${kind} temperature must be between 0.0 and 1.0. Got ${temperature}.`,
    );
  }
  const maxTokens = config.maxTokens ?? PROVIDER_DEFAULT_MAX_TOKENS;
  if (maxTokens < 1) {
    throw new ProviderConfigurationError(
      `[ArcFlow] ${kind} maxTokens must be at least 1. Got ${maxTokens}.`,
    );
  }
  return { maxTokens, temperature };
}

export class OpenAI {
  readonly model: string;
  readonly maxTokens: number;
  readonly temperature: number;

  constructor(config: ProviderConfig) {
    const validated = validateProvider("OpenAI", config);
    this.model = config.model.trim();
    this.maxTokens = validated.maxTokens;
    this.temperature = validated.temperature;
  }

  bindingRow() {
    return {
      kind: "openai",
      model: this.model,
      maxTokens: this.maxTokens,
      temperature: this.temperature,
    };
  }
}

export class Anthropic {
  readonly model: string;
  readonly maxTokens: number;
  readonly temperature: number;

  constructor(config: ProviderConfig) {
    const validated = validateProvider("Anthropic", config);
    this.model = config.model.trim();
    this.maxTokens = validated.maxTokens;
    this.temperature = validated.temperature;
  }

  bindingRow() {
    return {
      kind: "anthropic",
      model: this.model,
      maxTokens: this.maxTokens,
      temperature: this.temperature,
    };
  }
}

export class Gemini {
  readonly model: string;
  readonly maxTokens: number;
  readonly temperature: number;

  constructor(config: ProviderConfig) {
    const validated = validateProvider("Gemini", config);
    this.model = config.model.trim();
    this.maxTokens = validated.maxTokens;
    this.temperature = validated.temperature;
  }

  bindingRow() {
    return {
      kind: "gemini",
      model: this.model,
      maxTokens: this.maxTokens,
      temperature: this.temperature,
    };
  }
}

export type Provider = OpenAI | Anthropic | Gemini;
