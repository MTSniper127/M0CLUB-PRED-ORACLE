
import test from "node:test";
import assert from "node:assert/strict";
import { getJson } from "./_helpers.js";

test("e2e: commit-reveal endpoints exposed (read-only smoke)", async () => {
  // In this repository scaffold, commit/reveal is enforced on-chain,
  // but the gateway can still expose read-only epoch metadata.
  const epochs = await getJson<any[]>("/epochs");
  assert.ok(Array.isArray(epochs));
});
