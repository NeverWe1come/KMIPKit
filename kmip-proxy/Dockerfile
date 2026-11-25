# syntax=docker/dockerfile:1.7

ARG PROXY_VARIANT=all-clouds
ARG KMIPKIT_VERSION=1.0.0

# ------------------------------------------------------------------------------
# Optional local source build stage:
#   docker build --target local-runtime -f kmip-proxy/Dockerfile --build-arg PROXY_VARIANT=aws .
# ------------------------------------------------------------------------------
FROM rust:1.94-bookworm AS local-builder
ARG PROXY_VARIANT

RUN apt-get update && apt-get install -y --no-install-recommends \
    perl make gcc pkg-config python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .

RUN python3 verification/security/openssl/build.py \
    && OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9" \
       OPENSSL_STATIC=1 \
       cargo build --locked --release --manifest-path kmip-proxy/Cargo.toml --features "${PROXY_VARIANT}" \
    && cp kmip-proxy/target/release/kmipkit-proxy /usr/local/bin/kmipkit-proxy

FROM ubuntu:24.04 AS local-runtime
ARG PROXY_VARIANT
ARG KMIPKIT_VERSION

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

LABEL org.opencontainers.image.title="kmipkit-proxy (${PROXY_VARIANT})" \
      org.opencontainers.image.description="Zero-DB Stateless Cloud KMS Bridge for KMIP 2.0, 2.1, and 3.0" \
      org.opencontainers.image.version="${KMIPKIT_VERSION}" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/NeverWe1come/KMIPKit"

COPY --from=local-builder /usr/local/bin/kmipkit-proxy /usr/local/bin/kmipkit-proxy
EXPOSE 5696
USER ubuntu
ENTRYPOINT ["/usr/local/bin/kmipkit-proxy"]

# ------------------------------------------------------------------------------
# Default CI multi-arch runtime stage (uses pre-built static-OpenSSL binaries):
#   docker buildx build --platform linux/amd64,linux/arm64 \
#     -f kmip-proxy/Dockerfile --build-arg PROXY_VARIANT=aws <staging-context>
# ------------------------------------------------------------------------------
FROM ubuntu:24.04 AS ci-runtime
ARG TARGETARCH
ARG PROXY_VARIANT
ARG KMIPKIT_VERSION

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

LABEL org.opencontainers.image.title="kmipkit-proxy (${PROXY_VARIANT})" \
      org.opencontainers.image.description="Zero-DB Stateless Cloud KMS Bridge for KMIP 2.0, 2.1, and 3.0" \
      org.opencontainers.image.version="${KMIPKIT_VERSION}" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/NeverWe1come/KMIPKit"

COPY ${PROXY_VARIANT}/${TARGETARCH}/kmipkit-proxy /usr/local/bin/kmipkit-proxy
EXPOSE 5696
USER ubuntu
ENTRYPOINT ["/usr/local/bin/kmipkit-proxy"]
