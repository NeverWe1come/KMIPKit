/**
 * @mainpage KMIPKit C11 ABI Reference
 *
 * @section intro_sec Overview
 * **KMIPKit** exposes a stable, versioned **C11 ABI** (`libkmipkit_ffi`) backed by a
 * memory-safe Rust core supporting **OASIS KMIP 2.0, 2.1, and 3.0**.
 *
 * @section memory_sec Memory Ownership & Zeroization
 * - Every opaque handle returned by `kmipkit_*` is an 8-round Feistel-permuted token.
 * - Never pass pointers returned by KMIPKit to `stdlib.h` `free()`. Always invoke the
 *   matching `kmipkit_<type>_free()` destructor, which zeroizes secret buffers.
 * - Every exported function catches Rust panics and maps them to `KMIPKIT_ERROR_INTERNAL (7)`.
 *
 * @section groups_sec API Modules
 * - @ref kmipkit_version "Runtime Version & Capabilities"
 * - @ref kmipkit_errors "Thread-Local Error Snapshots"
 * - @ref kmipkit_transport "mTLS Raw TLS & HTTPS Transport Configuration"
 * - @ref kmipkit_client "Synchronous Client & Batch Execution"
 * - @ref kmipkit_fluent "Fluent Operation Builders"
 */

/** @defgroup kmipkit_version Runtime Version & Capabilities */
/** @defgroup kmipkit_errors Thread-Local Error Snapshots */
/** @defgroup kmipkit_transport mTLS Raw TLS & HTTPS Transport Configuration */
/** @defgroup kmipkit_client Synchronous Client & Batch Execution */
/** @defgroup kmipkit_fluent Fluent Operation Builders */
