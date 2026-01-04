
FROM rust:1.78-slim as builder
WORKDIR /app
COPY . /app
RUN cargo build -p m0-realtime --release

FROM debian:bookworm-slim
RUN useradd -m appuser
WORKDIR /app
COPY --from=builder /app/target/release/m0-realtime /usr/local/bin/m0-realtime
USER appuser
EXPOSE 8090
ENTRYPOINT ["m0-realtime"]
