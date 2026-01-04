
FROM rust:1.78-slim as builder
WORKDIR /app
COPY . /app
RUN cargo build -p m0-jobs --release

FROM debian:bookworm-slim
RUN useradd -m appuser
WORKDIR /app
COPY --from=builder /app/target/release/m0-jobs /usr/local/bin/m0-jobs
USER appuser
ENTRYPOINT ["m0-jobs"]
