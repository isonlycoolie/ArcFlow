import { randomUUID } from "node:crypto";

import { WorkflowConfigurationError } from "./exceptions.js";

export interface AgentConfig {
  name: string;
  role: string;
  instructions: string;
  model?: string;
}

export class Agent {
  readonly name: string;
  readonly role: string;
  readonly instructions: string;
  readonly model: string;
  readonly agentId: string;

  constructor(config: AgentConfig) {
    this.name = requireNonEmpty("name", config.name);
    this.role = requireNonEmpty("role", config.role);
    this.instructions = requireNonEmpty("instructions", config.instructions);
    this.model = (config.model ?? "default").trim() || "default";
    this.agentId = randomUUID();
  }

  bindingRow(): { id: string; name: string; role: string; instructions: string } {
    return {
      id: this.agentId,
      name: this.name,
      role: this.role,
      instructions: this.instructions,
    };
  }
}

function requireNonEmpty(field: string, value: string): string {
  const trimmed = value.trim();
  if (!trimmed) {
    throw new WorkflowConfigurationError(
      `[ArcFlow] Agent ${field} must be a non-empty string. ` +
        `Provide a meaningful ${field} for this agent.`,
    );
  }
  return trimmed;
}
