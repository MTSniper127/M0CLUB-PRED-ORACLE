
# M0Club Services

This folder contains runtime services:
- api-gateway (HTTP REST)
- realtime (WebSocket pubsub)
- indexer (Solana log poller + parsers)
- jobs (periodic maintenance tasks)
- dashboard (Next.js UI)

Local development (quick):
```bash
cd services

# Rust services
cargo build --workspace
cargo run -p m0-api-gateway -- --bind 127.0.0.1:8080
cargo run -p m0-realtime -- --bind 127.0.0.1:8090
cargo run -p m0-indexer -- --rpc-url https://api.devnet.solana.com
cargo run -p m0-jobs -- --interval-secs 15

# Dashboard (separate terminal)
cd dashboard
npm install
NEXT_PUBLIC_API_BASE=http://localhost:8080 NEXT_PUBLIC_WS_URL=ws://localhost:8090/ws npm run dev
```
