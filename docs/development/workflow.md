# Agent and specification workflow

## Principles

- Spec Driven Development determines what to build.
- Test Driven Development determines how implementation proceeds.
- A human approves specifications and PRs.
- Agents execute bounded, reviewed specifications.
- Shared foundations stabilize before parallel feature work.

## Spec Kit

Spec Kit is installed in this repository. The maintainer owns its
configuration. The workflow is:

1. Constitution.
2. Specify.
3. Plan.
4. Tasks.
5. Implement.

Specification identifiers use `KMIPKIT-NNNN`. Each specification has one
verifiable outcome, normative references, acceptance criteria, exclusions,
dependencies, security considerations, tests, and documentation effects.

A specification is split when it contains independently useful outcomes or
requires unrelated modules. Dependent specifications execute in order.

## Foundation phase

One continuous agent develops the foundation so that shared contracts do not
diverge. Foundation acceptance includes:

- Workspace and CI.
- Normative catalog and generation.
- Common types, errors, and unknown value policy.
- Generic TTLV model, encoder, and decoder.
- Message/batch base model.
- Transport abstraction and fake transport.
- Functional raw TLS and HTTPS path.
- Synchronous client request/response vertical slice.
- Minimal C ABI vertical slice.
- Coverage and conformance harness.

After human acceptance, independent agents may implement specifications in
parallel when they do not share unstable modules.

## Branch and worktree flow

```text
master
  release/1.0.0
    feature/KMIPKIT-0001-example
    feature/KMIPKIT-0002-example
```

- `master` remains stable and publishable.
- A release branch integrates one target version.
- A feature branch starts from the active release branch.
- Each agent uses a dedicated worktree.
- Feature PRs target the release branch and squash merge.
- The release branch integrates into master and receives the signed version
  tag.

## TDD evidence

1. Red commit: tests fail for the expected missing behavior.
2. Green commit: minimum implementation makes them pass.
3. Refactor commits: design improves while tests stay green.
4. Verification: focused, affected, then complete checks.

The PR records commands, expected failure reason, results, and final evidence.
The permanent verification record maps requirements to tests. Raw logs are
kept only when needed to explain a failure.

## Agent handoff

A draft PR must contain:

- Summary and rationale.
- Specification and requirement IDs.
- TDD evidence.
- Tests and platforms executed.
- Coverage and compatibility results.
- Security and secret-handling effects.
- Generated file changes.
- Risks, limitations, and follow-up work.

The agent may update its branch and draft PR. A human reviews, approves, and
merges. The agent cannot alter the approved scope to make completion easier.

## Change discovered during implementation

- A missing implementation detail within the approved design is resolved and
  documented.
- A changed requirement, public contract, security boundary, or scope pauses
  affected work.
- Update the specification or create an ADR and obtain human approval.
- Resume TDD only after the authoritative document is approved.

## Definition of done

The shared definition of done is in `AGENTS.md`. CI will eventually encode the
machine-verifiable portion; human review remains responsible for correctness,
scope, architecture, clarity, and security judgment.
