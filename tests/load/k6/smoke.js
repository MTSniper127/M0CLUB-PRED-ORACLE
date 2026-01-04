
import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  vus: 5,
  duration: "15s"
};

const BASE = __ENV.M0_API_BASE || "http://localhost:8080";

export default function () {
  const r1 = http.get(`${BASE}/health`);
  check(r1, { "health ok": (r) => r.status === 200 });

  const r2 = http.get(`${BASE}/markets`);
  check(r2, { "markets ok": (r) => r.status === 200 });

  sleep(0.5);
}
