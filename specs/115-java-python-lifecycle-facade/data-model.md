# KMIPKIT-0115 Data Model

## Operation Builder

- Holds one client reference, an operation kind, typed operation inputs,
  optional exchange-local `RequestOptions`, and an optional asynchronous
  Request Header indicator.
- `build`/`toBatchItem` creates a fresh typed batch item and does not send it.
- `execute` creates a one-item batch and uses the existing client exactly once.

## Locate Builder

- Requires an `AttributeSet`/typed attribute structure, which may be empty.
- Optional values: maximum items, offset items, storage-status mask, and
  object-group-member selector.
- Missing optional values remain missing in the request.

## Revoke Builder

- Requires a typed Unique Identifier and explicit reason code.
- Optional values: reason message and compromise-occurrence date.
- Copies message data into the existing native owner during build and follows
  the runtime's existing limits on clearing external copies.

## Result

- Uses existing typed batch responses; success-only result accessors remain
  absent for Pending and Failure outcomes.
- Delivery state and correlation remain those produced by the current client.
