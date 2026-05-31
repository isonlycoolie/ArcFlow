export function newId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  throw new Error("[ArcFlow] crypto.randomUUID is unavailable in this environment.");
}
