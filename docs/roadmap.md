# KMIPKit roadmap

This roadmap records implementation order and release work. Feature code is
added from approved Spec Kit specifications. “Integrated” means merged into
the reviewed release branch; it does not mean every conformance, interoperability,
security, package, or publication gate has passed.

**Status snapshot:** `release/1.0.0` at `ef9a19b770b59d29af1770a1c99e7634e7ff9034`
(2026-11-10). See the [project status](project-status.md) for evidence details.

## Phase 0: repository governance — integrated

Constitution and Spec Kit, community and security policies, ADR process,
contribution rules, CI, CODEOWNERS, DCO, release/feature branch workflow, and
AI agent rules are present. Repository protection and release evidence are
rechecked by applicable CI workflows.

## Phase 1: shared foundation — integrated

The workspace, OASIS source inventory and generated catalog, generic ordered
TTLV model and bounded codec, protocol errors and validation, message and
batch models, transport contract and fake, production TLS/HTTPS path,
synchronous client execution, C ABI foundation, and quality automation are
present in the release branch. Foundation stabilization no longer blocks
independent feature work; each feature still needs its own approved
specification and worktree.

## Phase 2: KMIP protocol and operations — integrated, evidence tracked

Typed objects, attributes, credentials, extensions, message models,
asynchronous protocol outcomes, and operation request/response surfaces have
been added through approved features. The catalog and conformance record remain
authoritative for requirement-level closure; a complete API surface alone does
not establish profile conformance.

## Phase 3: public API and language parity — integrated, release gate

The Rust facade, C ABI, Java JNI API, and Python CFFI API have undergone
cross-language parity work through KMIPKIT-0043 and KMIPKIT-0047–0053. The
versioned API manifest and generators define the public surface. Package
publishing and final compatibility evidence remain separate release gates.

## Phase 4: production transport and packages — implementation integrated, packaging evidence pending

Raw TLS and HTTPS/HTTP 1.1, TLS 1.3 mutual authentication, request delivery
states, TLS credential reload, session resumption, and keep-alive reconnect
are implemented under the approved transport specifications. JSON and XML
codecs and client/transport integration are implemented under KMIPKIT-0054–0057.
KMIPKIT-0068 tracks package dry-runs; the dated status page records its PR state
at the snapshot.

## Phase 5: typed API usability and preflight validation — integrated

Named protocol values, typed attributes and managed objects, operation
parameters, request authentication, schema-aware pre-send validation, typed
responses, generic response navigation, and parity across the four supported
languages are integrated through KMIPKIT-0047–0053.

## Phase 6: protocol breadth, interoperability, and release hardening — implementation integrated; release evidence remains

- KMIP 2.1 server-to-client operations and role handling: KMIPKIT-0066 and
  ADR-0021.
- KMIP 2.0 and catalogued KMIP 3.0 version support: KMIPKIT-0069–0071 and
  ADR-0022. The KMIP 3.0 surface remains limited to the pinned draft and
  approved feature requirements; it is not a blanket conformance claim.
- Shared interoperability catalog and Rust, C, Java, and Python validators:
  KMIPKIT-0058–0062. The server compatibility matrix remains an evidence
  ledger; untested operations and products stay pending.
- Standalone multi-version test server: KMIPKIT-0065.
- Release work still includes final package build/install/smoke tests, complete
  release-matrix checks, profile evidence, independent security review,
  signed artifacts, hashes, SBOM, provenance, release notes, and human release
  approval. Consult `docs/development/testing.md` and
  `docs/development/git-and-releases.md` for current gates.

The transport benchmark in `docs/development/transport-performance.md` is a
reproducible local HTTPS loopback baseline. It is not a cross-platform service
target; controlled regression analysis requires comparable machine and toolchain
records.

## Separate deliverable: stateless KMS proxy

The proxy has its own Cargo workspace, dependency boundary, specifications,
and provider workflows. KMIPKIT-0072–0082 cover the foundation and bridges for
AWS, Azure, Google Cloud, Oracle OCI, Vault/OpenBao, Infisical, IBM, Alibaba,
Scaleway, and Open Telekom Cloud. Its implementation does not make proxy
support part of the core client library or its 1.0 packages. See ADR-0023.

## Later roadmap candidates

These remain subject to approved specifications, compatibility review, and
the security policy:

- Additional idiomatic adapters: Go, C++, C#, and JavaScript.
- Rust async/await APIs.
- Connection pools and capability caching.
- Additional TLS identity providers and supported platforms.
- Declarative extension SDKs and optional compiled extension packages.
- Further transport and operational facilities.

KMIP 1.x client support is excluded by ADR-0022. Automatic KMIP request
retries remain prohibited by `AGENTS.md`; any proposed explicit retry facility
must not silently replay a request and requires a reviewed specification and
security decision. Proxy support is already tracked as the separate deliverable
above, not an unowned future core-client item.
