
import fetch from "node-fetch";

export const BASE = (process.env.M0_API_BASE ?? "http://localhost:8080").replace(/\/$/, "");

export async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, { headers: { accept: "application/json" } });
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`HTTP ${res.status} ${path}: ${body}`);
  }
  return (await res.json()) as T;
}

export async function sleep(ms: number): Promise<void> {
  await new Promise((r) => setTimeout(r, ms));
}
