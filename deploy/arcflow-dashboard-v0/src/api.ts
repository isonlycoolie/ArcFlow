const base = () => import.meta.env.VITE_ARCFLOW_ADMIN_URL as string;
const key = () => import.meta.env.VITE_ARCFLOW_ADMIN_KEY as string;

async function admin<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${base()}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      "X-ArcFlow-Admin-Key": key(),
      ...(init?.headers ?? {}),
    },
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`[ArcFlow] ${res.status}: ${text}`);
  }
  return res.json() as Promise<T>;
}

export type SiteCreated = {
  site_id: string;
  relay_url: string;
  site_token: string;
};

export function createSite(displayName: string, origin: string) {
  return admin<SiteCreated>("/v1/admin/sites", {
    method: "POST",
    body: JSON.stringify({
      display_name: displayName,
      allowed_origins: [origin],
    }),
  });
}

export function ingestKnowledge(siteId: string, text: string, docKey: string) {
  return admin<{ chunks_ingested: number }>(
    `/v1/admin/sites/${siteId}/knowledge/ingest`,
    { method: "POST", body: JSON.stringify({ text, key: docKey }) },
  );
}

export function publishChat(siteId: string, instructions: string) {
  return admin<{ name: string; version: string }>(
    `/v1/admin/sites/${siteId}/workflows/chat/publish`,
    { method: "POST", body: JSON.stringify({ instructions }) },
  );
}

export function patchSite(siteId: string, allowedOrigins: string[]) {
  return admin(`/v1/admin/sites/${siteId}`, {
    method: "PATCH",
    body: JSON.stringify({ allowed_origins: allowedOrigins }),
  });
}
