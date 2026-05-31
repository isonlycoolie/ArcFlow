import { useState } from "react";

export function useAsyncRun() {
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function run<T>(fn: () => Promise<T>) {
    setLoading(true);
    setError(null);
    try {
      await fn();
      setStatus("Done.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  return { status, error, loading, run };
}
