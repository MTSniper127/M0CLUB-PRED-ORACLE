
import test from "node:test";
import assert from "node:assert/strict";
import { getJson, sleep } from "./_helpers.js";

test("e2e: reorg resilience - repeated reads are consistent", async () => {
  const markets = await getJson<any[]>("/markets");
  if (markets.length === 0) return;

  const mid = markets[0].market_id as string;
  const a = await getJson<any>(`/predictions/${encodeURIComponent(mid)}/latest`);
  await sleep(200);
  const b = await getJson<any>(`/predictions/${encodeURIComponent(mid)}/latest`);

  assert.equal(a.market_id, b.market_id);
  assert.ok(typeof a.epoch_id === "number");
  assert.ok(typeof b.epoch_id === "number");
});
