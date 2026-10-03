# Coding standards

## General

- Code and identifiers are English.
- Prefer clear, concrete names.
- One module has one coherent responsibility.
- Avoid duplication of knowledge, especially protocol tables and validation.
- Apply KISS, DRY, SRP, and SOLID as reasoning tools, not slogans.
- Introduce patterns only when they make extension, testing, or invariants
  easier to understand.
- Document workarounds, business decisions, and technical limits.

## Rust

- Edition 2024; MSRV 1.94.
- Official Rust style and rustfmt.
- `clippy::all` and `clippy::pedantic`; warnings denied.
- Narrow documented lint exceptions.
- No unsafe outside `kmipkit-ffi`.
- No panic-based control flow in production.
- Explicit error categories and source chains.
- Public APIs documented with examples where useful.
- Use newtypes for validated domain values.
- Use non-exhaustive public enums and retain unknown numeric values.
- Avoid hidden allocation and copies on sensitive paths.
- Prefer owned zeroizing storage for secrets over unsafe zero-copy tricks.

## Generated code

- The normative catalog and public API manifest are reviewed source files.
- Generators are versioned and deterministic.
- Generated files state their source and command.
- Generated files are committed but never edited manually.
- CI regenerates and requires a clean diff.
- Generated behavior is tested even when raw generated lines are excluded from
  coverage.

## C ABI

- C11.
- Fixed-width integer types.
- Opaque handles.
- Explicit pointer/length pairs and nullability.
- No allocator crossing.
- Stable prefixed error codes and symbols.
- All unsafe blocks include a `SAFETY` argument.
- Every exported function contains panic protection.

## Java

- Java 17 source compatibility.
- Gradle Kotlin DSL.
- Idiomatic builders and immutable value types.
- Explicit JSpecify nullness.
- AutoCloseable resources.
- Unchecked typed exception hierarchy.
- Generated repetitive code and manually designed facade.

## Python

- Python 3.12 minimum.
- Full type annotations and `py.typed`.
- Ruff formatting/linting and Pyright.
- Context-managed native resources.
- Python naming and exception conventions.
- Generated repetitive code and manually designed facade.

## Documentation and comments

Rustdoc, Javadoc, docstrings, examples, ADRs, and specifications are English.
Comments explain why, invariants, safety, and non-obvious constraints. They do
not restate readable code.

## Dependency rule

Every new dependency needs a PR rationale covering capability, alternatives,
maintenance, security history, MSRV, license, platform support, and transitive
cost. Published builds contain no Git dependencies.
