# Contract: TLS Configuration

## Construction

- Accept one endpoint and one explicit trust source: caller CA certificates
  or an explicit platform-trust selection.
- Platform trust loads native root certificates with `rustls-native-certs`
  and validates them with rustls/WebPKI. When present, `SSL_CERT_FILE` takes
  precedence over platform-store loading for this trust mode; the selected
  environment-provided bundle is therefore part of the caller's explicit
  platform-trust configuration. KMIPKit fails closed if root loading reports
  an error. This choice does not
  import OS-specific distrust or revocation decisions. Callers requiring
  those decisions must supply explicit trust/CRL inputs within this feature's
  supported policy.
- Require a client identity. Accept PEM or DER only when selected explicitly;
  accept unencrypted PKCS#8, PKCS#1, or SEC1 private keys supported by AWS-LC.
- Read each selected file exactly once during construction. Resolve paths by
  normal OS rules. Keep parsed material, not paths, after successful
  construction.
- Reject encrypted keys, passphrase callbacks, empty or ambiguous key input,
  malformed certificates/CRLs, missing identity, and missing trust source.
- Construction validates configuration and starts an idle worker but opens
  no socket and performs no DNS request or KMIP operation.

## TLS behavior

- Configure rustls with an explicit AWS-LC provider and TLS 1.3 as the sole
  protocol version. Do not install or mutate a process-global provider.
- Require server-chain, certificate-time, and hostname/SAN validation. An
  optional separate server name is used only for certificate verification
  when the endpoint host is an IP address.
- Present the configured client certificate chain and private key for mTLS.
- Do not enable 0-RTT or key logging. Support session resumption only in
  bounded state owned by one client configuration; never persist/share
  sessions.
- A session cache is scoped to exactly one client configuration and holds at
  most 16 entries. Expire each cached ticket after one hour of local monotonic
  time or earlier if the server-provided ticket lifetime expires. A different
  identity/configuration cannot resume that session; no early application
  data is sent on a resumed or full handshake.
- A resumed session inherits the peer identity, certificate-time, hostname,
  trust-root, and caller-CRL decisions from its full handshake. KMIPKit does
  not revalidate the server certificate or re-evaluate CRLs during resumption.
  Caller trust inputs are loaded once at construction; rebuilding the client
  is required to apply changed trust inputs, and its session cache starts
  empty. This makes the authentication snapshot valid for at most one hour
  after its full handshake.
- Add only caller-supplied CRLs. Do not fetch CRL or OCSP data from the
  network. Fail closed when configured revocation evidence is invalid.

## Secret handling

Private-key source buffers and KMIPKit-owned temporary key bytes use explicit
zeroizing owners. Debug, Display, tracing, and public errors omit bytes,
passphrases, raw KMIP bodies, and untrusted dependency error text. Document
that caller copies and rustls/OS/dependency copies, including Hyper HTTP body
and parser buffers, are outside KMIPKit's zeroization guarantee. Error
metadata is restricted to a safe cause category, phase, delivery state, and
transport kind; endpoint host/path/query and credential paths are excluded.

## Public API boundary

The typed client is built from this validated configuration and KMIPKit's
production transport. Its constructor does not accept a caller-provided
`Transport`, raw bytes, or generic TTLV input. The documented low-level
`kmipkit-transport::Transport::exchange` remains a distinct direct Rust
caller-byte interface under ADR-0014. Each concrete production adapter also
exposes an additive `exchange_with_options` method accepting shared
`RequestOptions`; the existing `exchange` method uses the client timeout
policy. The direct options-bearing method is tested separately from the typed
client's per-operation variants.
