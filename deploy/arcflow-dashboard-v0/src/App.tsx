import { useState } from "react";
import { createSite, ingestKnowledge, publishChat, patchSite } from "./api";
import { useAsyncRun } from "./useAsyncRun";

export default function App() {
  const [siteId, setSiteId] = useState("");
  const [relayUrl, setRelayUrl] = useState("");
  const [siteToken, setSiteToken] = useState("");
  const [origin, setOrigin] = useState("http://localhost:5173");
  const [kbText, setKbText] = useState("");
  const [instructions, setInstructions] = useState(
    "Answer using the knowledge base. Be concise.",
  );
  const { status, error, loading, run } = useAsyncRun();

  return (
    <main style={{ fontFamily: "system-ui", maxWidth: 720, margin: "2rem auto" }}>
      <h1>ArcFlow Dashboard v0</h1>
      <p>Admin API: {import.meta.env.VITE_ARCFLOW_ADMIN_URL}</p>

      <section>
        <h2>Site</h2>
        <label>
          Allowed origin{" "}
          <input value={origin} onChange={(e) => setOrigin(e.target.value)} />
        </label>
        <button
          disabled={loading}
          onClick={() =>
            run(async () => {
              const s = await createSite("Dashboard site", origin);
              setSiteId(s.site_id);
              setRelayUrl(s.relay_url);
              setSiteToken(s.site_token);
            })
          }
        >
          Create site
        </button>
        {siteId && (
          <pre style={{ background: "#111", color: "#0f0", padding: "1rem" }}>
            {`VITE_ARCFLOW_RELAY_URL=${relayUrl}\nVITE_ARCFLOW_SITE_TOKEN=${siteToken}`}
          </pre>
        )}
        {siteId && (
          <button
            disabled={loading}
            onClick={() => run(() => patchSite(siteId, [origin]))}
          >
            Save origins
          </button>
        )}
      </section>

      <section>
        <h2>Knowledge</h2>
        <textarea
          rows={6}
          style={{ width: "100%" }}
          value={kbText}
          onChange={(e) => setKbText(e.target.value)}
        />
        <button
          disabled={loading || !siteId}
          onClick={() => run(() => ingestKnowledge(siteId, kbText, "faq"))}
        >
          Ingest
        </button>
      </section>

      <section>
        <h2>Chat</h2>
        <textarea
          rows={3}
          style={{ width: "100%" }}
          value={instructions}
          onChange={(e) => setInstructions(e.target.value)}
        />
        <button
          disabled={loading || !siteId}
          onClick={() => run(() => publishChat(siteId, instructions))}
        >
          Publish chat workflow
        </button>
      </section>

      {status && <p>{status}</p>}
      {error && <p style={{ color: "crimson" }}>{error}</p>}
    </main>
  );
}
