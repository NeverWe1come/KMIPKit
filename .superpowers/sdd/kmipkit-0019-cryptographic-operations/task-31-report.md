# Task 31 — Typed Encrypt/Decrypt Client Dispatch

Date: 2026-10-10

Branch: feature/KMIPKIT-0019-encrypt-decrypt-implementation

## Outcome

Implemented the minimum typed client plumbing needed to execute the focused T027, T028, and T029 contracts. Green implementation commit: 3e83024547767f8097857050aa595293ba00029b (DCO signed).

ClientRequest now carries typed Encrypt and Decrypt models, serializes their payloads with operation identifiers 0x0000001F and 0x00000020, and routes their response items through the existing result association, error, Pending, and delivery-state machinery. Typed response access is available through ClientResponseView and ClientBatchOutcome. No payload bytes are exposed through the typed request client; request Debug output is redacted.

The client validates only the existing Encrypt/Decrypt Data and multipart-indicator shape before consuming the request for serialization. That check reuses the protocol model’s existing multipart validator. Only failures from this local shape validation are mapped to Validation / InvalidInput / NotSent. Other payload-serialization ProtocolError values keep their existing classification. The rejection path runs before transport and returns the generic sanitized display:
client validation failure (invalid input, NotSent)

The ID Placeholder gate runs in the existing batch validation pipeline. For Encrypt and Decrypt requests with omitted Unique Identifier, it requires a structurally eligible preceding producer and effective Batch Order Option true. An omitted option remains omitted on the wire and uses KMIPKIT-0006’s existing effective default. The gate does not infer producer success, resolve or synthesize an identifier, reorder items, or move request fields. Create Split Key is not accepted as a producer.

Two additional boundary contracts were committed as a separate Red commit, d16cd3338b5d7fb7fcbbf4cb3c0eda97384226b0 (DCO signed): omitted Batch Order Option uses the existing true default, and Create Split Key does not establish ID Placeholder eligibility.

## Normative sources and model mapping

- The pinned OASIS KMIP v2.1 source, specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html, §6.1, lines 22397–22407, defines the ID Placeholder producers and permits subsequent batch operations to omit Unique Identifier when Batch Order Option is true or unspecified.
- Of modeled operations, Create maps to Create; Create Pair maps to Create Key Pair; Recover maps to Recover. Create Split Key does not appear in the §6.1 producer list and is explicitly excluded.
- KMIPKIT-0006 FR-006 in specs/006-message-batch-model/spec.md:115 specifies that an absent Batch Order Option has effective value True and its absence is preserved.
- The operation enumeration values are recorded in specification/catalog/kmip-2.1.json:41267–41287 for Decrypt (00000020) and :41411–41431 for Encrypt (0000001F). The client model constants are in crates/kmipkit-client/src/execute.rs:82–83. Their payload structures are Decrypt §6.1.11/Table 196 and Encrypt §6.1.17/Table 214.
- The Batch Order Option tag is catalog element KMIPKIT-ELEM-TAG-420010 at specification/catalog/kmip-2.1.json:65892–65902, and the protocol model constant is crates/kmipkit-protocol/src/message/header.rs:13 (0x0042_0010).
- The Request Header tag is catalog element KMIPKIT-ELEM-TAG-420077 at specification/catalog/kmip-2.1.json:69152–69161, and the client framing model is crates/kmipkit-client/src/execute.rs:63 (0x0042_0077). The T028 test constants were corrected from 0x0042_0078 (Message) and 0x0042_000B to the catalog/model values for Request Header and Batch Order Option.
- Architecture ruling: expose a read-only validate_multipart_shape method on each existing typed protocol request. The client crate cannot call the protocol crate’s private payload validator, and serializing first would consume the request and classify the failure as a generic payload ProtocolError. The new methods delegate to the same validator used during serialization and allow only this local Encrypt/Decrypt shape failure to be classified as Validation / NotSent.

## Red evidence

Before production edits, ran each existing focused client filter through WSL Ubuntu-26.04 in offline mode:

- wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_multipart_execution_tests --offline'
- wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_id_placeholder_execution_tests --offline'
- wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib client_accepts_framed_single_part_with_data_for_encrypt_and_decrypt --offline'
- wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib client_rejects_framed_single_part_without_data_before_exchange --offline'

Each stopped at test compilation with the expected missing API E0599 diagnostics: ClientRequest Encrypt/Decrypt variants and ClientResponseView encrypt/decrypt accessors. The focused reports document 14 missing API diagnostics across these contracts and no unrelated compile diagnostics; runtime tests did not run at Red. The additional T028 boundary tests were also re-run before Green and remained compile-Red for the same missing client APIs.

## Green evidence

All Rust commands ran in WSL Ubuntu-26.04 against the assigned worktree, offline:

- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo fmt --all --check' — passed.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_multipart_execution_tests --offline --message-format short' — 4 passed, 0 failed.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_id_placeholder_execution_tests --offline --message-format short' — 7 passed, 0 failed.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib client_accepts_framed_single_part_with_data_for_encrypt_and_decrypt --offline --message-format short' — 1 passed, 0 failed.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib client_rejects_framed_single_part_without_data_before_exchange --offline --message-format short' — 1 passed, 0 failed.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests --offline --message-format short' — 2 passed, 0 failed.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings' — passed.
- git diff --check — passed.

The T029 accepted form performed one exchange and preserved caller-supplied Data. The rejected Data-omission form asserted the exact generic error, Validation / InvalidInput / NotSent, and zero exchanges. The exact display contains no Data, AAD, or Tag names or request bytes. The T028 contracts preserved request item IDs and per-item server results when responses were reversed and when the Create producer failed; the default-option case preserved the omitted header field.


## Refactor evidence

Round 1 QA approved the behavior and required distinct Refactor evidence. Refactor commit: 3ea2cfabdaea6c63f67279e93b2eb36436a7d4d4 (DCO signed).

Changed ID Placeholder eligibility from rescanning every preceding batch prefix for each omitted-ID consumer to one ordered pass with an eligible-producer-seen flag. The pass checks each consumer before recording the current item, so only a preceding item establishes eligibility. It retains the same effective Batch Order Option default, producer matcher, consumer matcher, and IneligibleIdPlaceholder error. The worst-case scan is now linear in batch size instead of quadratic for batches with many omitted-ID consumers. No test, contract, or API changed.

Verification commands and results, run after the refactor in WSL Ubuntu-26.04, offline:

- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo fmt --all --check' — exit 0.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_id_placeholder_execution_tests --offline --message-format short' — exit 0; 7 passed, 0 failed, 289 filtered out.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo clippy -p kmipkit-client --all-targets --all-features --offline -- -D warnings' — exit 0.
- git diff --check — exit 0.

## T031 security fix round 1 — Pending Encrypt/Decrypt

Security reviewer finding: response_outcome sent Encrypt and Decrypt through their completed-only converters before the shared read_operation_outcome helper could inspect the result status. Those converters correctly returned PendingOutcomeRequired, so a structurally valid Operation Pending response was surfaced as a ProtocolError even though the client had already validated the asynchronous indicator and correlation value.

Ruling: retain the completed converters’ PendingOutcomeRequired behavior. Add explicit public pending-only response conversions that validate operation, shared operation-result metadata, and Operation Pending status, then construct the typed response with every success-only field absent. A pending-only conversion used with a non-Pending status returns the payload-free NotPendingOutcome error. The client checks item.result_status() before selecting either converter. It routes valid Pending responses through the existing pending_outcome path, which remains the only owner-copy point for the Asynchronous Correlation Value and stores it in Zeroizing<Vec<u8>>.

Red test commits, both DCO signed:
- 5de04e4a7f5e3eec8ff986833d86a4e71059a2c4 — added client Pending Encrypt/Decrypt contracts and protocol pending-only conversion contracts.
- 9fe3616b3dba7997bd18098ce2e522609f98d191 — corrected the protocol negative assertions to match the non-PartialEq response result types.

Red command and result:
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_multipart_execution_tests --offline --message-format short' — after correcting the fixture to include the structurally required empty Response Payload, 4 passed and the two new Pending cases failed with Protocol / InvalidValue / ResponseStarted because the completed-only converter rejected Pending.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib pending_encrypt_response_shape_tests --offline --message-format short' — expected compile-Red before the new API, with six E0599 diagnostics for missing pending-only converters and NotPendingOutcome variants.
- The first version of the client fixture omitted Response Payload and was correctly rejected by shared message-shape validation before dispatch. The committed fixture carries an empty Structure so Red reaches the Pending routing defect.

Green commit, DCO signed: 746a9edb2db7aee33a13694720512311b7f406ff.

Green implementation:
- Added EncryptResponse::try_from_pending_response_item and DecryptResponse::try_from_pending_response_item. They validate operation and the shared result/status, reject completed statuses with NotPendingOutcome, ignore the operation response payload, and initialize every success-only field to None.
- Preserved both completed converters’ PendingOutcomeRequired errors.
- Routed Pending before the completed converter and used pending_outcome for the exact opaque correlation bytes; the existing zeroization owner and generic payload-free error path remain in use.

Refactor commit, DCO signed: 59d0d9aa294926de7b977802138187247dc4a430. Extracted the shared Encrypt/Decrypt status dispatch into read_crypto_operation_outcome so both operations select the pending-only converter before the completed converter through one code path. This is behavior-preserving and keeps the two operation arms from diverging.

Post-Refactor verification commands and outputs, offline through WSL Ubuntu-26.04:
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_multipart_execution_tests --offline --message-format short' — exit 0; 6 passed, 0 failed, 292 filtered out.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib pending_encrypt_response_shape_tests --offline --message-format short' — exit 0; 7 passed, 0 failed, 312 filtered out. This includes checks that completed converters still reject Pending and pending-only converters reject completed statuses.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib pending_correlation_owner_zeroizes_before_release --offline --message-format short' — exit 0; 1 passed, 0 failed, 297 filtered out.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings' — exit 0.
- wsl.exe -d Ubuntu-26.04 -- bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo fmt --all --check' — exit 0.
- git diff --check — exit 0.

The new client tests assert Pending outcome, operation, shared result, exact correlation bytes containing NUL and non-UTF-8 values, typed response access, and absent success-only payload fields. No retries, polling, correlation synthesis, or change to the shared Pending lifecycle was introduced.
## Scope and known limitations

No T032 or T033 work was started. This change does not execute the 28 fixture-derived request/response pairs; T038 owns that work. The full FR-008 one-exchange-per-invocation acceptance remains for T034/T038 as recorded in the approved task brief. No OASIS upstream source, generated artifact, dependency, transport, retry, Poll behavior, batch ordering policy, or hidden multipart state was changed. No automatic field movement or correlation synthesis was added.
