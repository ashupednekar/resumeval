FROM rust:1.85-slim as builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y musl-tools && \
    rustup target add aarch64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --target aarch64-unknown-linux-musl

FROM gcr.io/distroless/cc-debian12:nonroot-arm64

COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/resumeval /resumeval

USER nonroot

ENTRYPOINT ["/resumeval"]
