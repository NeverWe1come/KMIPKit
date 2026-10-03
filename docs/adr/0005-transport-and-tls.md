# ADR-0005: Synchronous transports and TLS policy

Status: Accepted
Date: 2026-10-03

## Context

KMIP 2.1 clients need secure interoperable transport. The first API is
synchronous, and rustls does not implement the mandatory legacy TLS 1.2 RSA/CBC
suites required when a KMIP implementation elects to support TLS 1.2.

## Decision

Support TLS 1.3 only with rustls and aws-lc-rs. Use `TcpStream + rustls` for
raw TTLV and `reqwest::blocking + rustls` for HTTPS/HTTP 1.1. Require mTLS and
server verification. Disable 0-RTT, redirects, compression, proxies, key
logging, automatic retries, and automatic failover.

## Consequences

The transport is modern, memory safe, and consistently configurable. Products
that advertise KMIP 2.1 but only offer TLS 1.2 cannot connect to 1.0. A future
transport backend may address such legacy deployments without weakening the
default.

## Alternatives considered

OpenSSL increases platform and unsafe dependency complexity. Implementing HTTP
manually adds parser risk. Ureq currently exposes less precise stable TLS
version control. An async runtime would complicate the synchronous contract.
