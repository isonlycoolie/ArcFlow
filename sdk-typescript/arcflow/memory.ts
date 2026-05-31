/** Vector memory SDK surface (Phase 2-Pro). Mirrors Python `VectorStore`. */

export interface ChunkHit {
  text: string;
  byteLen: number;
}

type NativeVectorStore = {
  ingest(namespace: string, key: string, text: string): number;
  search(namespace: string, query: string, topK: number): string[];
};

function loadNativeVectorStore(): new () => NativeVectorStore {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const native = require("../index.native.js") as { JsVectorStore: new () => NativeVectorStore };
  return native.JsVectorStore;
}

export class VectorStore {
  private readonly native: NativeVectorStore;

  constructor() {
    const Native = loadNativeVectorStore();
    this.native = new Native();
  }

  ingest(namespace: string, key: string, text: string): number {
    const ns = namespace.trim();
    if (!ns) {
      throw new Error("[ArcFlow] VectorStore.ingest requires namespace.");
    }
    return this.native.ingest(ns, key, text);
  }

  search(namespace: string, query: string, topK = 5): ChunkHit[] {
    const ns = namespace.trim();
    if (!ns) {
      throw new Error("[ArcFlow] VectorStore.search requires namespace.");
    }
    return this.native.search(ns, query, topK).map((text) => ({
      text,
      byteLen: text.length,
    }));
  }
}
