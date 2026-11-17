# syntax=docker/dockerfile:1
FROM rust:1.94-bookworm@sha256:6ae102bdbf528294bc79ad6e1fae682f6f7c2a6e6621506ba959f9685b308a55 AS builder

ARG CLOUD_FEATURE=all-clouds
ARG PROXY_PROVIDERS=all-clouds
ARG KMIPKIT_VERSION=1.0.0

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
RUN case "$CLOUD_FEATURE" in \
      all-clouds|aws|azure|gcp|oci|vault|infisical|ibm|alibaba|scaleway|otc) ;; \
      *) echo "unsupported CLOUD_FEATURE: $CLOUD_FEATURE" >&2; exit 2 ;; \
    esac \
    && cargo build --locked --release --manifest-path proxy/kmipkit-proxy/Cargo.toml --no-default-features --features "$CLOUD_FEATURE"

# Minimal runtime image
FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc

ARG KMIPKIT_VERSION=1.0.0
ARG PROXY_PROVIDERS=all-clouds

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --user-group --shell /bin/false kmip

COPY --from=builder /usr/src/kmipkit/proxy/kmipkit-proxy/target/release/kmipkit-proxy /usr/local/bin/kmipkit-proxy

USER kmip

LABEL org.opencontainers.image.version="$KMIPKIT_VERSION" \
      org.opencontainers.image.title="KMIPKit Proxy" \
      io.kmipkit.proxy.providers="$PROXY_PROVIDERS"

EXPOSE 5696

ENTRYPOINT ["/usr/local/bin/kmipkit-proxy"]
CMD ["--config", "/etc/kmipkit/proxy.toml"]
