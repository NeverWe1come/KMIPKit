# Production transport performance baseline

This benchmark records a reproducible local baseline for configuration and
the production HTTPS transport. It uses only an in-process loopback peer with
ephemeral certificates; it does not contact a KMIP server or the network.

## Run

```text
cargo bench --locked -p kmipkit-transport --bench production_transport -- --iterations 40
```

The benchmark runs five warmup samples before each reported series. It reports
the nearest-rank p50 and p95 across the requested number of timed samples.
Pass a different positive sample count with `--iterations N`. Measurements
are built with Cargo's optimized `bench` profile and use the locked workspace
dependencies. The ephemeral PKI is generated once per run and excluded from
the timed samples.

## Measurements

| Measurement | Timed work |
|---|---|
| `TransportConfig::build` | Parse and validate a new public HTTPS configuration, including client identity and trust inputs. |
| `HttpsTransport::new` | Construct the production adapter from a prepared configuration; no DNS lookup or socket is opened. |
| TLS 1.3 mTLS handshake | A fresh TCP connection is established before timing. The timer covers the complete TLS 1.3 handshake with client-certificate verification. TLS session resumption is disabled for this series. No HTTP or KMIP exchange is included. |
| HTTPS cold first exchange | First production exchange on a newly constructed adapter. Includes lazy per-client worker start, loopback TCP connection, full mTLS handshake, and one HTTP/1.1 request/response. Adapter construction is outside the timer. |
| HTTPS reused steady-state exchange | One HTTP/1.1 request/response on a warmed adapter's persistent TLS connection. |
| Concurrent cold first exchange | First exchange from 1, 4, or 16 independently constructed adapters, dispatched on separate caller threads. Batch latency includes each adapter's lazy worker, TCP and TLS setup, the exchange, and caller-thread scheduling. |
| Concurrent reused steady-state batch | One concurrent request from each of 1, 4, or 16 warmed adapters. Connections and per-client workers are reused. The reported duration is for the whole batch, not an individual request. |

All exchanges use the shared request sentinel and an eight-byte response body.
The local peer requires a valid client certificate and verifies the client
certificate chain. These results characterize this machine and loopback setup;
they are not a cross-platform performance guarantee or a remote-server SLA.

## Baseline results

Record the command output verbatim for each environment, together with the
environment fields below. Do not compare results from different machines or
build profiles as if they were a controlled regression series.

| Environment | Rust | Iterations | Results |
|---|---|---:|---|
| Pending measurement | | | |

Environment record:

- OS and version:
- CPU model and logical processor count:
- Rust toolchain (`rustc -Vv`):
- Cargo profile and target triple:
- Commit:
- Power plan / CPU governor, if known:

## Interpretation

The cold exchange and concurrent cold batches include the production worker's
lazy startup, but do not isolate worker CPU time from TLS, socket, scheduling,
or request processing. Use the separately measured TLS handshake and adapter
construction series to distinguish those costs. Steady-state measurements
exclude initial connection setup and show the effect of HTTPS connection reuse.
