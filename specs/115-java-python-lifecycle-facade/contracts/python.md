# Python API Contract

`kmipkit.fluent.Request` adds `activate`, `archive`, `locate`, `recover`, and
`revoke`. Builders expose `validate`, `build`/`to_batch_item`, `with_options`,
an asynchronous indicator setter, and one-exchange `execute`. Revoke copies
the typed reason's code and message and accepts an optional named
`compromise_occurrence_date`.

`kmipkit.client.Client` adds keyword-friendly direct methods
`activate_by_id`, `archive_by_id`, `locate`, `recover_by_id`, and
`revoke_by_id`. Exchange options are passed by the named `options` parameter.
Locate takes required `attributes` and named optional filters. Revoke takes an
explicit typed identifier and `reason`, plus a named optional
`compromise_occurrence_date`. Existing public names and behavior remain source
compatible.
