# KMIPKIT-0007 dependency review

## `syn` Rust AST parser for OD-005

### Proposed selection

```toml
[dev-dependencies]
syn = { version = "=2.0.119", default-features = false, features = ["full", "parsing", "visit"] }
```

The dependency is test-only and implements the approved pinned Rust-AST audit for T008/T011 and OD-005. It does not enter the client production dependency graph. `full` and `parsing` are required for `syn::parse_file` to produce complete `syn::File` syntax trees; `visit` enables recursive AST traversal. Syn represents macro bodies as token streams and does not expand macros; the audit must separately handle or reject macro forms, aliases, `cfg` paths, modules, and any syntax it cannot inspect. Syn alone is not a proof about macro-expanded or semantically resolved Rust code.

### Package, compatibility, and provenance

- Exact version: `2.0.119`, checksum `872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297` (already present in `Cargo.lock` as a transitive package).
- Published: 2026-07-15; not yanked as checked on 2026-10-06.
- License: `MIT OR Apache-2.0`.
- Declared MSRV: Rust 1.71, compatible with the workspace MSRV 1.94.
- Upstream: <https://github.com/dtolnay/syn>.
- The latest 2.x version observed was 2.0.119; crates.io listed 3.0.6 as the latest overall release on 2026-10-06. No long-term support promise was found. Reassess the exact pin during dependency updates.

Primary package references:

- [Syn 2.0.119 `parse_file`](https://docs.rs/syn/2.0.119/syn/fn.parse_file.html)
- [Syn 2.0.119 feature definitions](https://docs.rs/crate/syn/2.0.119/source/Cargo.toml)
- [Syn 2.0.119 visitor API](https://docs.rs/syn/2.0.119/syn/visit/index.html)
- [Syn 2.0.119 manifest](https://docs.rs/crate/syn/2.0.119/source/Cargo.toml)
- [Syn 2.0.119 crates.io version record](https://crates.io/api/v1/crates/syn/2.0.119)

### Feature and package graph impact

The existing workspace test graph already enables Syn's default features (`clone-impls`, `derive`, `parsing`, `printing`, and `proc-macro`) through test dependencies including `trybuild` and `serde_derive`. The proposed direct client dev-dependency adds the client-to-Syn dev-only edge and adds the `full` and `visit` features to the workspace feature union. Its minimal feature selection does not activate optional printing, proc-macro, or quote support. The package dependencies and nodes (`syn`, `proc-macro2`, and `unicode-ident`) already exist in `Cargo.lock`; the direct edge adds no package node and no normal client dependency.

The implementation must verify this with locked Cargo metadata/tree output and a lockfile diff. Do not rely on transitive reachability as a substitute for the direct manifest edge.

### Security and maintenance review

An independent dependency reviewer checked the published manifest, exact lock checksum, feature behavior, compatibility, and workspace dependency tree. An exact-version OSV query for `syn` 2.0.119 returned an empty vulnerability list on 2026-10-06; the RustSec advisory database lookup for `crates/syn` returned 404. These dated database results are not proof that the package is free of unreported or undisclosed issues. Syn is a syntax parser, not a macro expander or semantic compiler; audit coverage remains the responsibility of the repository's fail-closed checker and adversarial fixtures.

Security references:

- [OSV API](https://api.osv.dev/v1/query), exact query package `syn` / ecosystem `crates.io` / version `2.0.119`, checked 2026-10-06.
- [RustSec advisory database package lookup](https://api.github.com/repos/RustSec/advisory-db/contents/crates/syn), returned 404 on 2026-10-06.

### Independent review and delegated maintainer disposition

The independent agent dependency review recommended accepting the exact dev-only selection above. The delegated maintainer disposition accepts it for OD-005 because it uses the lockfile's current Syn 2.x version, the minimal `full`/`parsing`/`visit` feature set, and no new package node or production dependency. This disposition authorizes only the parser dependency; it does not approve the AST audit implementation or its coverage. Security and QA must review the actual checker and fixtures before T011/T012 can pass.
