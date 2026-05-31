import { StaticConfigurationError } from "./errors.js";

/** Schema-only tool for static sites — execution happens on the server. */
export class Tool {
  readonly name: string;
  readonly description: string;
  readonly inputSchema: Record<string, unknown>;
  readonly permissions: string[] | undefined;

  constructor(options: {
    name: string;
    description: string;
    inputSchema: Record<string, unknown>;
    permissions?: string[];
  }) {
    const name = options.name.trim();
    if (!name) {
      throw new StaticConfigurationError("[ArcFlow] Tool name must be non-empty.");
    }
    this.name = name;
    this.description = options.description.trim();
    this.inputSchema = options.inputSchema;
    this.permissions = options.permissions;
  }

  toJson(): Record<string, unknown> {
    return {
      name: this.name,
      input_schema: this.inputSchema,
      permissions: this.permissions ?? null,
    };
  }
}
