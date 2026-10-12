# Rustdoc JSON format 57 fixture

`rustdoc_v57_fixture.json` is a minimal projection of real output from Rust
1.94.0 rustdoc. The source is `lib.rs`; it exercises inherent/trait member
name collisions, required and provided trait members, a derived external-trait
implementation, type aliases and alias chains, generic fields, and enum
payloads.

The source was captured with:

```text
rustdoc +1.94.0 --edition 2024 --crate-name rustdoc_v57_fixture --crate-type lib --output-format json -Z unstable-options --document-private-items --document-hidden-items lib.rs --out-dir <temporary-directory>
```

The checked-in JSON preserves rustdoc's actual format-57 item records and
relationships for the fixture surface, while omitting unrelated standard
library and compiler-generated implementation records to keep the test input
small. The `format_version` and `includes_private` fields remain the original
rustdoc values. Do not hand-author item shapes in this fixture; regenerate it
from the source and project only the required records when its source changes.
