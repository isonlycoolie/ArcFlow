import { ContextPolicy, ToolExecutionConfig } from "./context.js";
import { StaticConfigurationError } from "./errors.js";
import { newId } from "./ids.js";
import { MemoryConfig } from "./memory.js";
import { Tool } from "./tool.js";

export interface AgentConfig {
  name: string;
  role: string;
  instructions: string;
  memory?: MemoryConfig;
  tools?: Tool[];
  context?: ContextPolicy;
  toolExecution?: ToolExecutionConfig;
}

export class Agent {
  readonly name: string;
  readonly role: string;
  readonly instructions: string;
  readonly agentId: string;
  readonly memory: MemoryConfig | undefined;
  readonly tools: Tool[];
  readonly context: ContextPolicy | undefined;
  readonly toolExecution: ToolExecutionConfig | undefined;

  constructor(config: AgentConfig) {
    this.name = requireNonEmpty("name", config.name);
    this.role = requireNonEmpty("role", config.role);
    this.instructions = requireNonEmpty("instructions", config.instructions);
    this.agentId = newId();
    this.memory = config.memory;
    this.tools = config.tools ?? [];
    this.context = config.context;
    this.toolExecution = config.toolExecution;
  }

  bindingRow(): Record<string, unknown> {
    const row: Record<string, unknown> = {
      id: this.agentId,
      name: this.name,
      role: this.role,
      instructions: this.instructions,
    };
    if (this.memory) row.memory_config = this.memory.toJson();
    if (this.tools.length) row.tools = this.tools.map((t) => t.toJson());
    if (this.context) row.context = this.context.toJson();
    if (this.toolExecution) row.tool_execution = this.toolExecution.toJson();
    return row;
  }
}

function requireNonEmpty(field: string, value: string): string {
  const trimmed = value.trim();
  if (!trimmed) {
    throw new StaticConfigurationError(`[ArcFlow] Agent ${field} must be a non-empty string.`);
  }
  return trimmed;
}
