
FROM rust:1.78-slim as builder
WORKDIR /app
COPY . /app
RUN cargo build -p m0-indexer --release

FROM debian:bookworm-slim
RUN useradd -m appuser
WORKDIR /app
COPY --from=builder /app/target/release/m0-indexer /usr/local/bin/m0-indexer
USER appuser
ENTRYPOINT ["m0-indexer"]
