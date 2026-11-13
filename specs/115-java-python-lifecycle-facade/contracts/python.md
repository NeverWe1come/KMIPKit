# Python API Contract

`kmipkit.fluent.Request` adds `activate`, `archive`, `locate`, `recover`, and
`revoke`. Builders expose `validate`, `build`/`to_batch_item`, `with_options`,
an asynchronous indicator setter, and one-exchange `execute`.

`kmipkit.client.Client` adds keyword-friendly direct methods
`activate_by_id`, `archive_by_id`, `locate`, `recover_by_id`, and
`revoke_by_id`. Exchange options are passed by the named `options` parameter;
Locate filters and the optional Revoke date/message are named parameters.
Existing public names and behavior remain source compatible.
