# Java examples

[VendorExtensionExample.java](VendorExtensionExample.java) registers vendor
extension definitions, inspects and validates local TTLV values, and attaches
extensions to a Discover Versions batch item. It uses non-secret sample values
and performs no server exchange.

Run it through the Maven test lifecycle from the repository root:

```sh
mvn --batch-mode --file bindings/java/pom.xml test
```

The `run-vendor-extension-example` Maven execution invokes the example with the
built classes, dependency classpath, and explicit JNI library path. This command
also runs the binding tests; it requires the tools in
[installation](../docs/installation.md).

The example demonstrates explicit handle cleanup. Application-owned Java
strings and byte arrays need their own secret-handling policy. See
[native loading](../docs/native-library.md) and the
[shared architecture](../../../docs/architecture/ffi-and-bindings.md).
