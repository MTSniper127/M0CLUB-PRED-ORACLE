
# M0Club Test Suite

This folder contains:
- `integration/` TypeScript end-to-end tests (gateway + services)
- `load/` k6 load tests
- `fuzz/` fuzz scaffolds (programs + engine)

Local run (requires services running; see infra/docker compose.dev.yml):
```bash
cd tests
npm install
npm test
```

k6 smoke:
```bash
cd tests
M0_API_BASE=http://localhost:8080 npm run k6:smoke
```
