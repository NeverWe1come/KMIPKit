# Specification: Unified Multilanguage Documentation Platform and Native Client Manuals (KMIPKIT-0067)

## 1. Overview and Purpose

This specification governs the architecture, layout, generator tooling, and quality gates for the complete, bilingual, multi-language documentation ecosystem of `KMIPKit`.

To eliminate divergence across four target languages (Rust, C, Java, Python) and prevent duplication between internal architectural artifacts and user-facing manuals, this specification establishes:

1. **Two-Level Documentation Architecture**:
   - **Level 1 — Central Documentation Portal (`docs/site/`)**: An Astro Starlight portal deployed via GitHub Pages containing the complete encyclopedia: 3-minute quickstarts, deployment across platforms (Linux, macOS, Windows, Docker), testing with `kmipkit-test-server`, KMIP 1.0–2.1 protocol encyclopedia, TTLV wire framing, operations dictionary with synced polyglot code tabs, production cookbook (mTLS, batches, async, vendor extensions), HSM interoperability guides (Thales, Utimaco, Kryptus, Entrust, Vault, Fortanix), internal workspace architecture, security model, and live OASIS conformance matrices.
   - **Level 2 — Native Client Documentation (`docs/client-manuals/{rust,c,java,python}/`)**: Idiomatic, client-only manuals tailored specifically for standard language distribution hubs (`docs.rs`, `Doxygen`, `javadoc.io`, `Read the Docs` / `PyPI`), applying the Strict Separation Rule (zero internal workspace noise).

2. **Single Source of Truth (SSOT)**:
   - Operation tables, TTLV tags, enumerations, error reasons, and wire traces derive directly from checked-in machine-readable catalogs (`specification/catalog/kmip-2.1.json`, `specification/api/public-api.json`).
   - Architectural and security documents in `docs/` (`adr/`, `architecture/`, `security/`, `compliance/`) are imported into the site structure without manual copy-paste.

3. **Bilingual User Guide (Spanish and English)**:
   - Full bilingual parity across all 10 site modules in accordance with `constitution.md` and `AGENTS.md` §11.

## 2. Normative References

- **KMIPKit Constitution (`.specify/memory/constitution.md`)**: Sections I, III, IV, and Product Constraints.
- **KMIPKit Agent Rules (`AGENTS.md`)**: Section 2 (Product boundaries), Section 9 (Public API and compatibility), Section 10 (Generated artifacts), Section 11 (Documentation).
- **ADR-0001 to ADR-0020**: Layered workspace, C ABI, transport, public API, vendor extensions, conformance sources, and session resumption.
- **OASIS KMIP 2.1 Specification, Profiles, and Test Cases**: Authoritative protocol behavior.

## 3. Scope and Public API Impact

- **Public API impact**: 0% API or ABI breaking changes.
- **New directories and files**:
  - `docs/architecture/documentation-architecture.md`: Normative architecture document.
  - `docs/site/`: Astro Starlight web application source and bilingual markdown/MDX content.
  - `docs/client-manuals/`: Language-specific client manuals for Rust, C, Java, and Python.
  - `tools/docgen/`: Deterministic catalog-driven documentation generation script.
  - `.github/workflows/docs.yml`: Automated CI build, link verification, and deployment workflow.
  - `specs/067-multilanguage-documentation/`: Specification artifacts.

## 4. Acceptance Criteria

1. **`REQ-067-001` (Unified Directory Layout)**:
   All documentation, web portal sources, and native manual templates reside under `docs/` (`docs/site/` and `docs/client-manuals/`), leaving workspace root clean and preserving existing references in `constitution.md` and `AGENTS.md`.

2. **`REQ-067-002` (Two-Level Architecture and Strict Separation)**:
   - `docs/site/` contains all 10 modules covering the entire KMIPKit project and protocol.
   - `docs/client-manuals/` provides purely idiomatic client usage guides across the 6 standard sections (Installation, Quickstart, Connection & Transport, Client Methods, Error Handling & Secrets, Local Testing).

3. **`REQ-067-003` (Bilingual Content Parity)**:
   All 10 modules in `docs/site/` are authored in both Spanish (`es/`) and English (`en/`).

4. **`REQ-067-004` (Automated SSOT Generation Tool)**:
   `tools/docgen/` extracts TTLV tags, wire names, enumerations, operations, and attributes from `specification/catalog/kmip-2.1.json` and `specification/api/public-api.json`, generating reference tables and operations dictionary pages without manual drift.

5. **`REQ-067-005` (Astro Starlight Configuration)**:
   `docs/site/` includes Astro Starlight configuration with locale switching (`es`/`en`), synced language tabs (`Rust | C | Java | Python`), Mermaid diagram support, and Pagefind offline search.

6. **`REQ-067-006` (CI Workflow and Link Integrity)**:
   A GitHub Actions workflow verifies documentation build, validates link integrity, and ensures generated tables match the normative catalog.
