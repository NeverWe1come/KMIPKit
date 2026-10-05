# KMIPKIT-0005 delegated authorization record

**Recorded**: 2026-10-05
**Authorization source**: KMIPKit maintainer instruction in this Codex conversation
**Status**: Authorized for the bounded specification and implementation work below

## Authorization

On 2026-10-05, the maintainer directly instructed Codex to execute the
complete KMIPKIT-0005 plan autonomously without requesting further approval
or manual intervention, and later authorized creating the pull request from
the terminal. This is delegated maintainer authorization for the bounded
scope and decisions recorded here. The repository contract still reserves
human approval, merge, and publication. This record does not claim that the
maintainer separately reviewed each artifact or that an independent human
security audit has occurred.

## Approved scope and decisions

1. Approve `specs/005-ttlv-wire-codec/spec.md` and its implementation contract
   for a public bounded TTLV decoder and immutable per-call limits, plus a
   private `kmipkit-client` writer and zeroizing owner. KMIPKIT-0005 must not
   add `Client::execute`, a production writer callsite, or a permit.
2. Accept ADR-0011's KMIPKit policy to reject received Reserved Tags before
   generic Item construction. T001 records this as accepted KMIPKit policy decision
   `KMIPKIT-DEC-001`, links it to `KMIPKIT-DISC-037`, and cites OASIS KMIP
   Specification v2.1 §11.56 for the Tag-range source context before validating
   and regenerating the catalog report ahead of T006. Section 11.56 does not
   prescribe receiver rejection; this decision neither reinterprets nor
   deviates from `KMIPKIT-REQ-SPEC-11.56-001`. T012 later adds only applicable
   implementation and test traceability for normative requirements assigned
   to this codec; it does not defer or repeat the project-policy disposition.
3. Accept ADR-0012's narrow policy for temporary outbound TTLV that carries an
   explicitly caller-requested typed operation. In KMIPKIT-0005 this permits
   only a private, uncalled writer/owner. The first client feature/spec owns
   the closed typed request API, execute-owned permit, sole production mint
   and writer callsite, and owner-through-transport integration test. That
   candidate PR must include both the callsite and test; CI must pass the test
   against that candidate before merge, enablement, or any secret-bearing
   send. The release branch must contain no production callsite or
   secret-bearing send before that gate passes.
4. Resolve KMIPKIT-0005-OD-001 as bounded preflight and fallible
   decoder-owned scratch/payload reservations. Existing model allocations via
   `Box::new` and `Vec::push` may abort on OOM; this feature does not change
   those constructors and does not claim recovery from every allocation
   failure.
5. Accept the dependency-review dispositions in
   `dependency-review.md`: use the reviewed scopes and exact package
   selections, retain `zeroize` 1.9.0's performance caveat, and keep
   test-only Serde at 1.0.228 for current lock/tree alignment. Recheck the
   exact package graph and features before editing the client manifest.
6. Keep `KMIPKIT-REQ-SPEC-10.1.2-001` as an explicit global follow-on gap until
   every applicable client Structure has approved typed-spec ownership,
   implementation, and executable field-order verification. Do not claim
   complete roadmap traceability from KMIPKIT-0005 alone.

## Independent review evidence and boundaries

- The independent dependency review and delegated package dispositions are
  recorded in `dependency-review.md`.
- The independent QA review identified the prerequisite ordering gate
  (T001 must close `KMIPKIT-DISC-037` and regenerate the catalog report before
  T006), inline OASIS attribution for normative cases, safe test-only
  zeroization observation, and stale reviewer-owned checklist notes. This
  task records the ordering in the plan and tasks, updates the test-case
  requirements, and appends checklist reconciliation notes without changing
  reviewer-owned checkboxes.
- The independent security/design review found no clear mismatch in the cited
  OASIS KMIP 2.1 §§10.1.1–10.1.5, 11.23, and 11.56. It required preserving
  the existing `Box::new`/`Vec::push` OOM-abort limitation and the first-client
  production callsite plus candidate-CI integration gate; both remain explicit
  in the approved contract and plan.

These were independent agent reviews, not a qualified human security audit.
The roadmap's independent human security review before 1.0 remains required.
No exact upstream source under `specification/oasis/` is modified.

## Approved artifact revisions

The following SHA-256 digests identify the delegated scope snapshot and
its decision, catalog, generated-report, feature, and architecture
artifacts. They bind the worktree state recorded here; the digest for this
file is intentionally omitted to avoid a self-referential record. The feature branch is
`feature/KMIPKIT-0005-ttlv-wire-codec` at `10ccc99180760ff34778b35db9b99db81fef38f5`;
it contains active `release/1.0.0` head
`849b46f772fd7cbcea2f42393cdbcf58e92cac94` as an ancestor. The working tree
was verified against those refs on 2026-10-05. Task progress is included below.

## Execution update — 2026-10-05

After the maintainer reported that intervening PRs were merged, a terminal GitHub API check confirmed PRs #14–#29 were closed as merged. This feature branch integrated `origin/release/1.0.0` at `d46e13dfdd83ac05e4b58d24af5b928e355e092d` in merge commit `bd27ebc`. The catalog merge preserves `KMIPKIT-DEC-001` / `KMIPKIT-DISC-037` and the merged credential corrections and open `KMIPKIT-DISC-041` / `KMIPKIT-DISC-042`; the generated coverage report was regenerated from the validated catalog. The catalog regression test now accepts discrepancies resolved by an accepted decision while requiring unresolved rows to remain open. Validation, report verification, source audit, immutable-source check, generators, and all catalog tests pass. T003 and T004 are complete after clean scoped reviews; the T004 observer regression was corrected with separate Red/Green commits. These changes do not expand the approved scope. The artifact digests below reflect the merged release and completed T004 task state.

| Artifact | SHA-256 |
| --- | --- |
| `spec.md` | `01C75CE5DD645FDBC1ED227698ACBE3F54657BE4ABB13C4CFD63285DE5B87832` |
| `plan.md` | `BD027496A530B0D6C22A36CF1EFB7F25DFAA473D69929EEF5C97B3D526555294` |
| `research.md` | `2C519C123A47B01F1E1C7997EDF99584324A592E488EC8CB0EEF23991ABBECE6` |
| `data-model.md` | `5AAC5218210EDF98D37CB4AB5DB39A3D8B8E00F3D8ADF4E88AAE34553450AD95` |
| `contracts/rust-ttlv-codec.md` | `728BABDA1F12996AC47A617999E0FDF7FF0F884B87A73AE51C3D3509942F5CEA` |
| `quickstart.md` | `8AB455C58E907DA26931217CCC6DFA4EA0F45E34A5775FA8F74926E9883F85F1` |
| `dependency-review.md` | `60FE528D34DCB30F8F83BC6176B91E8522B4A6FA9CA1A5875B32697E9F0206B4` |
| `tasks.md` | `E8A9FB122CC504B15A97B754DF235AD97620B06230221FC5120CCF95A181947F` |
| `docs/adr/0011-reserved-tag-decoding-policy.md` | `3A5C0589554B2080835EC7F10F1D537B297CFF9B60F21E89F03869DE428A1D2A` |
| `docs/adr/0012-caller-requested-wire-encoding-policy.md` | `130C4CAD7AFC9985BC816286F8652729B392425C5E510DCAB9285223E827F333` |
| `docs/adr/README.md` | `EA965F505F893CAA505E6A783ECE6BF5488AE61B77D3E4A37192B2E8B34F4ECC` |
| `docs/architecture/overview.md` | `EB435926FB747C950B1D3418C436D91D70E5690D08B4F499183C9A0636337C55` |
| `docs/architecture/public-api.md` | `5F10D0CD4DA8C7EA012969795BED3AFDA62DD9DF88A7EAB251EC7B4561A26139` |
| `docs/adr/0010-tag-allocation-precedence.md` | `2A85D69BCE2D7CF3439E3FF4CE900BD1C6CF100DEDA37FDDB13292C5601E7473` |
| `specification/catalog/kmip-2.1.json` | `06D10D118F4D877033BD0D83EEC7C9345C350823149714236BAB2F6094694F53` |
| `specification/catalog/coverage-report.md` | `9C36100B581D9E94DEB3E842A5521639EB49649229A0013100462DAB6ECFA578` |

## Execution update — T005

T005 Red is complete in DCO-signed commit `0937633685d44aaa921e13aec3449d6cce319f40`. The decoder test module is private and test-only; all 33 Red cases compiled and failed at behavioral assertions against the intentionally incomplete seam. The independent review recorded PASS for scope and test quality with no blockers. It confirmed the assigned and extension Tag fixtures, OASIS attribution, and padding extent semantics; the reviewer did not rerun commands. The task report retains the command evidence and review limitations. T006 is now unblocked; its Green work will promote the private seam to the production decoder and public facade, preserving the approved one-item, bounded, payload-redacted contract.

| Updated artifact | SHA-256 |
| --- | --- |
| `tasks.md` | `4EA2A97243B9D025D79CA2BA05C348D859A1DF3BD31E0B50132D29CD8A717C89` |

This authorization does not authorize agents to approve or merge their PRs,
push to `master` or `release/*`, publish a release, or bypass the first-client
integration gate. Human approval and merge remain governed by the repository
constitution and `AGENTS.md`.
