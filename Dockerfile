# Build stage
FROM rust:1.85-slim AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY src ./src
COPY static ./static
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd --system appuser && useradd --system --gid appuser --create-home --home-dir /home/appuser appuser
WORKDIR /app
COPY --from=builder /app/target/release/gerador-senhas ./
COPY --from=builder /app/static ./static
RUN chown -R appuser:appuser /app /home/appuser
USER appuser
EXPOSE 3000
CMD ["./gerador-senhas"]
