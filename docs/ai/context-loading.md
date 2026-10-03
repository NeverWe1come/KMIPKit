# AI context loading guide

Giving an agent every KMIP document on every task reduces focus and increases
the chance of mixing normative and informative text. Load a stable base and
then only the task-relevant standard sections.

## Base context for every development task

1. `AGENTS.md`.
2. `docs/design/project-definition.md`.
3. `docs/compliance/document-hierarchy.md`.
4. The approved Spec Kit specification, plan, and tasks.
5. Relevant accepted ADRs.
6. Existing code and tests in the affected layers.

## Protocol task context

Add:

- Exact applicable sections from the pinned KMIP Specification.
- Applicable profile clauses.
- Relevant official test cases.
- Usage Guide passages only when they clarify intent.
- Existing catalog and traceability records.

Record document, stage, section, and normative level in the specification. Do
not paste unattributed fragments into a prompt.

## Transport or security task context

Add the applicable KMIP profile transport clauses, transport/security ADR,
threat model, dependency documentation, and existing integration tests. Treat
server data, certificates, HTTP, FFI inputs, manifests, and callbacks as
untrusted.

## Binding task context

Add the approved public API manifest, C ABI contract, language binding
architecture, ownership rules, and shared parity vectors. The adapter must not
invent protocol behavior.

## Generated-code task context

Add the catalog schema, generator source, representative input/output, and
compatibility baseline. Change reviewed input before output and verify
determinism.

## Agent output contract

The agent must identify:

- Specification and requirement IDs implemented.
- Files and public contracts changed.
- Red, Green, Refactor evidence.
- Verification commands and results.
- Traceability and documentation updates.
- Security, compatibility, and interoperability effects.
- Remaining limitations inside the approved scope.

## Prohibited context practices

- Treating an entire web search result as normative.
- Combining KMIP versions unless the task explicitly concerns comparison.
- Relying on memory for tag values or conformance language.
- Allowing Usage Guide examples to override the Specification.
- Asking an agent to implement an unapproved broad feature from one prompt.
- Supplying real credentials, private keys, or customer traffic captures.
