# C Native Library & Memory Ownership

1. **Always use matching destructors:** Never call `free()` from `stdlib.h` on pointers returned by `kmipkit_*`. Always call `kmipkit_<type>_free()`.
2. **Versioned structs:** Initialize `struct_size = sizeof(s)` and `abi_version = 1` before calling `kmipkit_last_error_get_v1()` or extension registry limit APIs.
3. **Thread-local error snapshots:** Call `kmipkit_last_error_get_v1(&snap)` after any non-`KMIPKIT_OK` status to inspect `category`, `cause`, and `delivery_state`, then `kmipkit_last_error_clear()`.
