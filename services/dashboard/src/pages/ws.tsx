
import { useEffect, useMemo, useRef, useState } from "react";

export default function WsPage() {
  const url = useMemo(() => (process.env.NEXT_PUBLIC_WS_URL ?? "ws://localhost:8090/ws"), []);
  const [logs, setLogs] = useState<string[]>([]);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      ws.send(JSON.stringify({ subscribe: "predictions" }));
      setLogs((l) => [`connected: ${url}`, ...l]);
    };

    ws.onmessage = (ev) => setLogs((l) => [String(ev.data), ...l].slice(0, 50));
    ws.onerror = () => setLogs((l) => ["error", ...l]);
    ws.onclose = () => setLogs((l) => ["closed", ...l]);

    return () => ws.close();
  }, [url]);

  return (
    <main style={{ padding: 24 }}>
      <h1>Realtime WS Demo</h1>
      <p>URL: {url}</p>
      <pre style={{ background: "#111", color: "#eee", padding: 12, borderRadius: 8, overflowX: "auto" }}>
{logs.join("\n")}
      </pre>
    </main>
  );
}
