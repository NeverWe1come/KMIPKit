# Java API Contract

Under `org.kmipkit.fluent.FluentRequest`:

- `activate(UniqueIdentifier)`, `archive(UniqueIdentifier)`, and
  `recover(UniqueIdentifier)` return an operation builder.
- `locate(AttributeSet)` returns a Locate builder with explicit optional
  filters.
- `revoke(UniqueIdentifier, RevocationReason)` returns a Revoke builder with
  message copied from the typed reason and an optional compromise-occurrence
  date set on the builder.
- Builders expose `validate`, `build`, `toBatchItem`, `withOptions`, an
  asynchronous indicator setter, and one-exchange `execute(CodecLimits)`.

`org.kmipkit.client.KmipClient` adds `activateById`, `archiveById`,
`recoverById`, `locate`, and `revokeById`. Each has a `(typed inputs,
CodecLimits)` overload and an overload with trailing `RequestOptions`; Revoke
also takes nullable `Long compromiseOccurrenceDate` before `CodecLimits`.
Existing method names and signatures remain source compatible.
