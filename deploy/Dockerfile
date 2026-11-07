# syntax=docker/dockerfile:1
FROM rust:1.85-bookworm AS builder

WORKDIR /usr/src/kmipkit

# Install build dependencies required for aws-lc-sys and C toolchain
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    clang \
    llvm \
    perl \
    make \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace source files
COPY . .

# Build kmipkit-proxy binary with all supported cloud providers
RUN cargo build --release --manifest-path proxy/kmipkit-proxy/Cargo.toml --features all-clouds

# Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --user-group --shell /bin/false kmip

COPY --from=builder /usr/src/kmipkit/proxy/kmipkit-proxy/target/release/kmipkit-proxy /usr/local/bin/kmipkit-proxy

USER kmip

EXPOSE 5696

ENTRYPOINT ["/usr/local/bin/kmipkit-proxy"]
CMD ["--config", "/etc/kmipkit/proxy.toml"]
