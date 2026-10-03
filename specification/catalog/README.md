# Normative catalog

This directory will contain the reviewed machine-readable catalog that drives
code generation. The exact schema is defined by its own approved
specification.

Every entry must include a stable ID, KMIP name, wire value, type, source
document and section, version, normative constraints, and extension rules as
applicable.

The catalog is a derived implementation source. OASIS remains authoritative.
Generated code is committed and never edited by hand. CI regenerates it and
requires a clean diff.
