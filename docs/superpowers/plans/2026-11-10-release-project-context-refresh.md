# Release Project Context Documentation Refresh Implementation Plan

> **For agentic workers:** This plan is being executed inline in the current session. Each step is tracked below.

**Goal:** Bring the release branch's entry-point and status documentation into agreement with approved scope, merged code, and current release evidence.

**Architecture:** Keep approved constitutions and ADRs authoritative. Add one dated project-status snapshot, then reconcile the README, project definition, roadmap, architecture, workspace guide, interoperability matrix, and index so readers can find current facts without interpreting contradictory historical text.

**Tech Stack:** Markdown, Git history, approved Spec Kit specifications, CI and pull-request metadata.

**Spec:** User request in this conversation; authoritative sources are `AGENTS.md`, `.specify/memory/constitution.md`, accepted ADRs, approved feature specifications, and the current `origin/release/1.0.0` code.

## Global Constraints

- Base all changes on `origin/release/1.0.0` at `ef9a19b770b59d29af1770a1c99e7634e7ff9034`.
- Do not change the constitution, accepted ADRs, specifications, generated files, or product behavior.
- Distinguish implemented code from passing evidence, formal profile claims, and publication.
- Preserve dated historical evidence and label snapshot-dependent status.
- This documentation-only change does not require code tests; review links, counts, and claims against their source documents.

---

### Task 1: Record the current release snapshot

**Files:** Create `docs/project-status.md`.

- [x] Record the reviewed commit and date, integrated capability areas, open release gates, and evidence limitations.

### Task 2: Reconcile project entry points

**Files:** Modify `README.md`, `docs/design/project-definition.md`, and `docs/README.md`.

- [x] Remove superseded scope and foundation-phase claims; link to the dated status snapshot.
- [x] Align 1.0 scope and exclusions with the constitution and accepted ADRs.
- [x] Make guides and project-status material discoverable.

### Task 3: Reconcile delivery and architecture descriptions

**Files:** Modify `docs/roadmap.md`, `docs/architecture/overview.md`, and `docs/development/rust-workspace.md`.

- [x] Mark delivered implementation areas and outstanding release evidence without claiming formal conformance.
- [x] Describe the current crates, production transport/client path, encodings, bindings, listeners, versions, and proxy boundary.

### Task 4: Correct the interoperability scope statement

**Files:** Modify `docs/development/kmip-interoperability.md`.

- [x] Update the 1.0 directional scope and testability legend; keep vendor test results explicitly pending unless recorded.

### Task 5: Review the documentation diff

- [x] Search modified pages for obsolete foundation, scope, crate-count, and transport statements.
- [x] Check Markdown links and the final diff; do not run implementation tests for documentation-only changes.
