# Pinned OASIS KMIP 2.1 sources

The `upstream/` files are exact HTML responses downloaded from official OASIS
URLs on 2026-10-03. They are reference inputs for humans and AI agents.

Do not edit, reformat, translate, or apply the KMIPKit license to these files.
Their original copyright and notices remain inside each document. KMIPKit is
an independent implementation and is not endorsed or certified by OASIS.

Use [`SOURCES.md`](SOURCES.md) for canonical URLs, document status, and supplemental fixture hashes. Use [`CHECKSUMS.sha256`](CHECKSUMS.sha256) to verify the immutable upstream source copies.

Exact linked OASIS XML test-case fixtures are stored in `fixtures/`, outside
the immutable `upstream/` HTML copies. Their source URLs, work-product
version/date, and SHA-256 are recorded in [`SOURCES.md`](SOURCES.md). Do not
modify fixture bytes.

The normative precedence rules are in
[`docs/compliance/document-hierarchy.md`](../../../docs/compliance/document-hierarchy.md).

When OASIS publishes an erratum or updated stage, add it as a new pinned input
through a reviewed specification. Never overwrite an existing source silently.
