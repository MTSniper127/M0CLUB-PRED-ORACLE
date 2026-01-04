
async function fetchJson(url: string) {
  try {
    const res = await fetch(url, { cache: "no-store" });
    return await res.json();
  } catch {
    return null;
  }
}

export default async function Page() {
  const base = process.env.NEXT_PUBLIC_API_BASE ?? "http://localhost:8080";
  const markets = await fetchJson(`${base}/markets`);

  return (
    <main style={{ padding: 24 }}>
      <h1 style={{ marginBottom: 8 }}>M0Club Dashboard</h1>
      <p style={{ marginTop: 0, opacity: 0.8 }}>
        Connects to API Gateway for markets and latest predictions.
      </p>

      <section style={{ marginTop: 24 }}>
        <h2>Markets</h2>
        <pre style={{ background: "#111", color: "#eee", padding: 12, borderRadius: 8, overflowX: "auto" }}>
{JSON.stringify(markets, null, 2)}
        </pre>
      </section>

      <section style={{ marginTop: 24 }}>
        <h2>WebSocket</h2>
        <p style={{ opacity: 0.8 }}>
          Use the client demo page under /pages/ws to subscribe to realtime updates.
        </p>
      </section>
    </main>
  );
}
