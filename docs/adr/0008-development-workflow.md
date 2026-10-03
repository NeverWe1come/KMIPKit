# ADR-0008: SDD, TDD, branches, and agents

Status: Accepted
Date: 2026-10-03

## Context

KMIP is large, normative, and security-sensitive. AI can accelerate work but
also amplify ambiguity and conflicting edits.

## Decision

Use Spec Kit SDD and strict Red, Green, Refactor TDD. Each approved
specification owns a feature branch, worktree, agent, tests, traceability, and
draft PR. A single agent builds the unstable foundation; independent agents
parallelize only after its acceptance. Humans approve and merge. Master remains
stable through release integration branches.

## Consequences

Work is auditable and parallelism is introduced only when boundaries are safe.
The process adds documentation and PR overhead that is accepted for protocol
correctness and community maintainability.

## Alternatives considered

Unspecified task prompts are faster initially but cannot demonstrate coverage.
Immediate parallel development would cause shared model conflicts. Agent merge
authority would remove the required human safety gate.
