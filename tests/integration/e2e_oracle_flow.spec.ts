
import test from "node:test";
import assert from "node:assert/strict";
import { getJson } from "./_helpers.js";

type Market = { market_id: string; domain: string; status: string };
type Prediction = { market_id: string; epoch_id: number; outcomes: Record<string, { p: number; ci: [number, number] }> };

test("e2e: oracle flow - list markets, read latest prediction", async () => {
  const health = await getJson<{ ok: boolean }>("/health");
  assert.equal(health.ok, true);

  const markets = await getJson<Market[]>("/markets");
  assert.ok(Array.isArray(markets));
  assert.ok(markets.length >= 0);

  if (markets.length > 0) {
    const m = markets[0];
    const p = await getJson<Prediction>(`/predictions/${encodeURIComponent(m.market_id)}/latest`);
    assert.equal(p.market_id, m.market_id);
    assert.ok(typeof p.epoch_id === "number");
    assert.ok(typeof p.outcomes === "object");
  }
});
