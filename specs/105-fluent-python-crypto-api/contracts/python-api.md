# KMIPKIT-0105 Python API Contract

`Client.request()` returns `kmipkit.fluent.Request`. Its methods construct
operation-specific builders: `encrypt()`, `decrypt()`, `sign()`,
`signature_verify()`, `mac()`, `mac_verify()`, and `hash()`.

Each builder provides only its approved setters plus common
`cryptographic_parameters()`, `cryptographic_parameters_structure()`,
`correlation_value()`, `init_indicator()`, `final_indicator()`,
`with_options()`, `asynchronous_indicator()`, `build()`, `to_batch_item()`,
`execute(limits)`, `close()`, and context-manager methods. Key-using operations
provide `unique_identifier()`. Operation-specific byte fields follow
`plan.md` and the KMIPKIT-0102 Rust field matrix.

`execute()` returns the current `kmipkit.client.BatchResult`, performs one
exchange, and does not retry or poll. The result's existing status and typed
response accessors remain authoritative.
