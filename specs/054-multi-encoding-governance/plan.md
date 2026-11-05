# KMIPKIT-0054 Implementation Plan

**Branch**: `feature/KMIPKIT-0054-multi-encoding-governance`
**Date**: 2026-11-05
**Spec**: [spec.md](spec.md)

## Summary

Record the approved 1.0 encoding scope amendment, resolve project handling of
the HTTPS profile text discrepancy for planning purposes, define the private
shared codec contract and catalog-derived tag/Enumeration/Bit Mask tables, and assign
independent JSON, XML, and integration specifications. Codec and client
implementation are assigned to 0055–0057.

## Contract and dependencies

- Shared semantic input/output is the existing ordered `kmipkit_ttlv::Item`
  model. Do not create a parallel JSON/XML value tree or a public alias.
- 0054 generates private tag names, Enumeration families and values, Bit Mask
  families and assigned components, and tag-to-family mappings offline from
  the reviewed catalog. JSON/XML codecs apply the matching OASIS name
  normalization and preserve unknown values using profile-permitted numeric
  or hexadecimal forms.
- Each decoder is private and applies existing message/depth/element limits
  before size-driven allocation. Errors remain payload-free and redacted.
- `WireEncoding`, configured default, per-exchange override, public dispatch,
  `public-api.json`, generated output, and language bindings are 0057 scope.
- 0055 JSON and 0056 XML depend on this accepted contract and may proceed
  independently. 0057 depends on the accepted contracts and implementations
  of both codecs.

## Documentation changes

- Amend the constitution, `AGENTS.md`, project definition, and roadmap to
  make the approved three-encoding 1.0 scope consistent.
- Supersede ADR-0002's encoding clause with ADR-0018 and record the HTTPS
  contextual interpretation without calling it an OASIS erratum.
- Update architecture overview and transport documentation with the common
  codec boundary, raw TLS document behavior, HTTPS content types, and the
  separate wire-byte and protocol response-size limits.
- Add approved specifications for 0055, 0056, and 0057, with corrected OASIS
  section ownership, bitmask representations, and the manifest edit boundary.

## Acceptance review

- Search all canonical product/governance documents for stale statements
  that 1.0 is TTLV-only or that JSON/XML are post-1.0.
- Verify every 0054 requirement maps to this spec and an owned documentation
  change; verify 0055/0056 point to JSON §5.5/XML §5.4 respectively.
- Verify no public API manifest/generated output, codec implementation, or
  client dispatch changed in 0054. Confirm private tag, Enumeration, and Bit
  Mask tables are reproducibly generated from the reviewed catalog.
- Run `git diff --check` and inspect the complete diff before review.

## Approval and implementation gates

The maintainer approved specifications 0054, 0055, 0056, and 0057, and
accepted ADR-0018 by direct instruction on 2026-11-05. Codec implementation
can proceed on the approved 0055 and 0056 worktrees after the common contract
is updated. Client API publication remains assigned to 0057. Independent
review, human PR approval/merge, and release gates remain required.
