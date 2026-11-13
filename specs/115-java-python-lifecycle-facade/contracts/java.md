# Java API Contract

Under `org.kmipkit.fluent.FluentRequest`:

- `activate(UniqueIdentifier)`, `archive(UniqueIdentifier)`, and
  `recover(UniqueIdentifier)` return an operation builder.
- `locate(AttributeSet)` returns a Locate builder with explicit optional
  filters.
- `revoke(UniqueIdentifier, RevocationReason)` returns a Revoke builder with
  optional message/date values sourced from the typed reason and builder.
- Builders expose `validate`, `build`, `toBatchItem`, `withOptions`, an
  asynchronous indicator setter, and one-exchange `execute(CodecLimits)`.

`org.kmipkit.client.KmipClient` adds named direct methods for the same five
operations, with exchange-local options overloads. Existing method names and
signatures remain source compatible.
