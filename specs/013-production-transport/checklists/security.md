# Security Requirements Checklist: Production TLS and HTTPS Transports

**Purpose**: Review whether the specification states complete and testable security properties for credentials, TLS, HTTP, parsing, lifecycle, and errors.
**Created**: 2026-10-07
**Feature**: [spec.md](../spec.md)

**Note**: This checklist tests security-requirement quality, not implementation behavior.
**Review Ownership**: Independent security reviewer. Leave unchecked until reviewed.

## Identity and Trust

- [ ] CHK001 Are hostname/SAN, validity period, chain, selected root source, TLS name, and mTLS identity requirements explicit and independently testable?
- [ ] CHK002 Does the contract forbid public certificate-verification bypasses, implicit roots, and silent TLS downgrade?
- [ ] CHK003 Are caller-supplied CRLs and the absence of online revocation fetching precisely scoped?
- [ ] CHK004 Are TLS 1.3-only, AWS-LC, disabled 0-RTT, disabled key logging, per-client ticket expiry, and inherited trust-snapshot semantics all explicit?

## Secrets and Memory

- [ ] CHK005 Are accepted private-key encodings, encrypted-key rejection, file read timing, symlink behavior, and caller file responsibilities explicit?
- [ ] CHK006 Does the spec distinguish KMIPKit-owned buffers from caller, TLS, OS, and third-party copies when describing zeroization?
- [ ] CHK007 Are initialized partial request/response buffers cleared on every failure path and constrained against unsafe reallocation after secret bytes are stored?
- [ ] CHK008 Are Debug, Display, errors, logs, tracing, and error chains prohibited from exposing keys, credentials, request/response bodies, or untrusted error text?

## Network and Parser Boundaries

- [ ] CHK009 Are all network bytes treated as untrusted and the raw TTLV header/length checked before allocation?
- [ ] CHK010 Does the HTTPS scope reject redirects, proxies, compression, unsupported encodings, ambiguous framing, and unexpected status without retry?
- [ ] CHK015 Are HTTP parser header-count and input-buffer bounds explicitly defined independently from the response-body limit?
- [ ] CHK011 Are response size and decoder depth/element limits enforced before growth or nested allocation?
- [ ] CHK012 Does the design define cancellation and teardown for the worker, resolver, TLS stream, and HTTP connection?
- [ ] CHK013 Does every timeout and protocol failure invalidate the connection and preserve delivery state without replaying the KMIP operation?
- [ ] CHK014 Does the resolver contract preserve OS-owned split-DNS/VPN policy, limit KMIPKit-submitted work with a shared governor, and avoid claims about OS packet retries, caching, or routing internals?
- [ ] CHK016 Does the delivery contract define a race-safe dispatch commit so `NotSent` can never be transmitted later, without guessing the HTTPS header/body boundary below Hyper?
- [ ] CHK017 Are platform-root loading semantics, `SSL_CERT_FILE`, lack of OS distrust/revocation handling, and caller-CRL behavior explicit and fail-closed?
- [ ] CHK018 Are TLS session resumption capacity, one-hour local expiry, same-client reuse, cross-identity isolation, inherited full-handshake trust decisions, rebuilt-client cache invalidation after trust changes, and 0-RTT disablement independently testable?
- [ ] CHK019 Are raw-TLS surplus frames prevented from being attributed to a later exchange, and are HTTPS parser-owned copies excluded accurately from zeroization claims?
- [ ] CHK020 Are the 32-permit governor scope, permit lifetime, fail-fast admission, 16-address cap, cancellation limits, late-result isolation, and background native-call effects stated and tested precisely?
- [ ] CHK021 Is `SSL_CERT_FILE` precedence tested in an isolated process and included in the platform-trust threat boundary?
- [ ] CHK022 Does the HTTP reuse test prevent an unsolicited response from being assigned to a later operation?
- [ ] CHK023 Is the race between first decrypted response-byte observation and timeout finalization resolved monotonically and tested?
- [ ] CHK024 Is exactly one HTTPS `Host` header derived from endpoint authority, preserving bracketed IPv6 literals and explicit non-default ports, independent of the TLS verification-name override?
- [ ] CHK025 Does the certificate-validation requirement distinguish a full handshake from resumed sessions, and is the inherited trust/CRL decision bounded by the stated ticket expiry?
- [ ] CHK026 Does a client extension provenance mismatch fail before request
  construction/encoding/exchange for `ClientRequestMessageExtension` with sanitized
  `InvalidInput`/`NotSent`, without exposing registry identity or extension payload data?

## Notes

- A completed checklist is independent of code review and test evidence.
- Do not claim a formal OASIS profile, FIPS status, or zeroization of external copies.
