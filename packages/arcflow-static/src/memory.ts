import { StaticConfigurationError } from "./errors.js";

export type MemoryTypeName = "Session" | "Shared" | "Persistent" | "Vector";
export type MemoryScopeName = "Agent" | "Workflow" | "Global";

export interface MemoryRetrievalOptions {
  mode?: "dense" | "hybrid";
  denseWeight?: number;
  sparseWeight?: number;
  rerank?: "cohere" | "local" | null;
  topK?: number | null;
}

export class MemoryRetrievalConfig {
  readonly mode: "dense" | "hybrid";
  readonly denseWeight: number;
  readonly sparseWeight: number;
  readonly rerank: "cohere" | "local" | null;
  readonly topK: number | null;

  constructor(options: MemoryRetrievalOptions = {}) {
    const mode = options.mode ?? "dense";
    if (mode !== "dense" && mode !== "hybrid") {
      throw new StaticConfigurationError("[ArcFlow] retrieval.mode must be 'dense' or 'hybrid'.");
    }
    this.mode = mode;
    this.denseWeight = options.denseWeight ?? 0.7;
    this.sparseWeight = options.sparseWeight ?? 0.3;
    this.rerank = options.rerank ?? null;
    this.topK = options.topK ?? null;
  }

  toJson(): Record<string, unknown> {
    return {
      mode: this.mode,
      dense_weight: this.denseWeight,
      sparse_weight: this.sparseWeight,
      rerank: this.rerank,
      top_k: this.topK,
    };
  }
}

export interface MemoryConfigOptions {
  type: MemoryTypeName;
  scope?: MemoryScopeName;
  namespace?: string;
  ttlSeconds?: number;
  embedding?: string;
  retrieval?: MemoryRetrievalConfig;
  chunking?: { strategy?: string; chunkSize?: number; overlap?: number };
}

export class MemoryConfig {
  readonly memoryType: MemoryTypeName;
  readonly scope: MemoryScopeName;
  readonly namespace: string | undefined;
  readonly ttlSeconds: number | undefined;
  readonly embedding: string | undefined;
  readonly retrieval: MemoryRetrievalConfig | undefined;
  readonly chunking: MemoryConfigOptions["chunking"];

  constructor(options: MemoryConfigOptions) {
    if (options.type === "Persistent" || options.type === "Vector") {
      const ns = (options.namespace ?? "").trim();
      if (!ns) {
        throw new StaticConfigurationError(
          "[ArcFlow] namespace is required for Persistent and Vector memory.",
        );
      }
      this.namespace = ns;
    } else {
      this.namespace = options.namespace?.trim() || undefined;
    }
    this.memoryType = options.type;
    this.scope = options.scope ?? "Agent";
    this.ttlSeconds = options.ttlSeconds;
    this.embedding = options.embedding?.trim() || undefined;
    this.retrieval = options.retrieval;
    this.chunking = options.chunking;
  }

  toJson(): Record<string, unknown> {
    const payload: Record<string, unknown> = {
      memory_type: this.memoryType,
      scope: this.scope,
      namespace: this.namespace ?? null,
      ttl_seconds: this.ttlSeconds ?? null,
    };
    if (this.embedding) payload.embedding = this.embedding;
    if (this.retrieval) payload.retrieval = this.retrieval.toJson();
    if (this.chunking) {
      payload.chunking = {
        strategy: this.chunking.strategy ?? "recursive",
        chunk_size: this.chunking.chunkSize ?? 512,
        overlap: this.chunking.overlap ?? 64,
      };
    }
    return payload;
  }
}
