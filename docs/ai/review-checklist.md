# AI change review checklist

Use this checklist for human review of an AI-authored PR.

## Scope and sources

- [ ] The PR implements one approved specification.
- [ ] OASIS citations identify exact pinned documents and sections.
- [ ] Normative and informative sources are distinguished.
- [ ] No unrelated refactor or unapproved capability is included.

## Correctness

- [ ] Types, ordering, required fields, and error behavior match KMIP 2.1.
- [ ] Unknown standard/vendor values remain lossless.
- [ ] Batch, pending, and correlation behavior is covered where applicable.
- [ ] Server policy is not encoded as a universal client restriction.

## TDD and evidence

- [ ] Red failure demonstrates missing behavior for the right reason.
- [ ] Green implementation is the minimum correct behavior.
- [ ] Refactoring preserves tests.
- [ ] Positive, negative, boundary, and property cases are appropriate.
- [ ] Coverage and traceability gates pass.

## Security

- [ ] Untrusted lengths and counts are bounded before allocation.
- [ ] Secrets cannot enter formatting, errors, logs, generated fixtures, or
      screenshots.
- [ ] Memory ownership and zeroization are explicit.
- [ ] No retry or ambiguous delivery is hidden.
- [ ] Unsafe/FFI code has a tight justified boundary.
- [ ] TLS verification and transport invariants remain intact.

## Compatibility

- [ ] Rust, C, Java, and Python parity is maintained when required.
- [ ] Public enums and structures remain extensible.
- [ ] ABI/API compatibility tools pass.
- [ ] Generated output matches reviewed inputs.
- [ ] Documentation and examples match behavior.

## Delivery

- [ ] PR description states risks and limitations plainly.
- [ ] CI covers applicable native platforms.
- [ ] No placeholder, ignored failure, or unexplained allow remains.
- [ ] A qualified human has reviewed sensitive changes.
