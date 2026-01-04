
import { spawnSync } from "node:child_process";

const scenarios = [
  { name: "smoke", file: "load/k6/smoke.js" }
];

for (const s of scenarios) {
  console.log(`Running scenario: ${s.name}`);
  const r = spawnSync("k6", ["run", s.file], { stdio: "inherit", env: process.env });
  if (r.status !== 0) process.exit(r.status ?? 1);
}
