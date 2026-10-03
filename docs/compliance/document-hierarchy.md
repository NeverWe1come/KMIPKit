# OASIS document hierarchy

Agents and reviewers use the following order when interpreting KMIP 2.1.

## 1. Normative computer language definitions

Any separate machine-readable content explicitly declared normative by OASIS
prevails over display content when OASIS states that rule for the work product.

## 2. KMIP Specification 2.1 OASIS Standard

The protocol specification defines messages, objects, attributes, operations,
encodings, rules, and conformance clauses. Applicable normative prose is the
primary human-readable source.

## 3. KMIP Profiles 2.1 OASIS Standard

Profiles add normative selections and requirements for a specific context.
They constrain use of the base specification; they do not redefine invalid
base protocol behavior.

## 4. Approved errata

Only official approved errata apply. Each is pinned, reviewed, and entered in
the conformance matrix. The repository never follows a mutable latest URL
silently.

## 5. KMIP Test Cases 2.1

Test cases are conformance evidence and examples. They help determine expected
bytes and workflows. An example does not create a requirement absent from a
normative source.

## 6. KMIP Usage Guide 2.1

The guide explains intent, common usage, and background. Treat it as
informative whenever it is not itself incorporated by a normative reference.

## 7. KMIPKit derived material

The catalog, generated code, ADRs, API docs, and commentary are implementation
artifacts. Every normative catalog entry must link upward to its OASIS source.
Derived content never overrides OASIS.

## Conflicts and ambiguity

Do not choose an interpretation silently. Record:

- The documents, stages, dates, and exact sections.
- Whether each statement is normative or informative.
- The observable alternatives.
- Compatibility and security effects.

Open a specification issue or ADR for human resolution. Add a conformance test
that preserves the resolved interpretation.

## Citation format

Every protocol implementation specification uses a stable requirement ID and
the official citation identifier plus section, for example:

```text
KMIPKIT-REQ-SPEC-8.3-001
Source: [kmip-spec-v2.1], section 8.3
Level: SHALL
Applicability: client request
```
