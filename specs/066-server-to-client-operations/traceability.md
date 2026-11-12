# Traceability Matrix: KMIPKIT-0066

| Requirement | Tasks | Verification |
| :--- | :--- | :--- |
| `REQ-066-001` (Set Endpoint Role C2S / S2C) | `TASK-066-002`, `TASK-066-009` | `crates/kmipkit-protocol/tests/unit/set_endpoint_role_tests.rs`; C listener exchange in `crates/kmipkit-ffi/tests/support/client_abi_tests.rs` |
| `REQ-066-002` (S2C Discover Versions) | `TASK-066-003` | `crates/kmipkit-protocol/tests/unit/s2c_discover_versions_tests.rs`; C listener exchange in `crates/kmipkit-ffi/tests/support/client_abi_tests.rs` |
| `REQ-066-003` (S2C Notify) | `TASK-066-004` | `crates/kmipkit-protocol/tests/unit/s2c_notify_tests.rs`; C changed attributes and deleted-reference access in `crates/kmipkit-ffi/tests/support/client_abi_tests.rs` |
| `REQ-066-004` (S2C Put) | `TASK-066-005` | `crates/kmipkit-protocol/tests/unit/s2c_put_tests.rs`; typed byte and credential-text secret access in `crates/kmipkit-ffi/tests/support/client_abi_tests.rs` |
| `REQ-066-005` (S2C Query & ResponseBuilder) | `TASK-066-006` | `crates/kmipkit-protocol/tests/unit/s2c_query_tests.rs`; C capability response construction in `crates/kmipkit-ffi/tests/support/client_abi_tests.rs` |
| `REQ-066-006` (mTLS Listeners) | `TASK-066-007` | `crates/kmipkit-transport/tests/listener_tests.rs`; C TLS 1.3 mTLS exchange in `crates/kmipkit-ffi/tests/support/client_abi_tests.rs` |
| `REQ-066-007` (Role-Reversed Stream) | `TASK-066-008` | `crates/kmipkit-transport/tests/role_reversal_tests.rs` |
| `REQ-066-008` (Test Server Active Trigger) | `TASK-066-011`, `TASK-066-012` | `crates/kmipkit-test-server/tests/s2c_trigger_tests.rs` |
| `REQ-066-009` (Four-Language Parity) | `TASK-066-013`, `TASK-066-014`, `TASK-066-015` | `bindings/c/tests/s2c_consumer.c`, `bindings/java/.../ServerListenerTest.java`, `bindings/python/tests/test_server_listener.py` |

### C binding callback boundary

The approved C secret accessors `kmipkit_secret_bytes_with_bytes` and
`kmipkit_secret_text_with_str` use `void` callbacks, so a callback cannot
return an application failure through this ABI. The C listener consumer checks
that a NULL callback is rejected with `KMIPKIT_ERROR_INVALID_INPUT` and that
the callback-scoped secret bytes are available only during the synchronous
call. This delivery does not add a callback result convention; that would
require an approved public ABI contract.
