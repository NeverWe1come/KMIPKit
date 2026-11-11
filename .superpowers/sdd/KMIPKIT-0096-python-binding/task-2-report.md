# KMIPKIT-0096 FR-003 review follow-up

Base: `7c4518ff0`, after the parent rebased T006 onto KMIPKIT-0095
`5791bcaddc60062d69310d37ad396e97ed30b69c`.

The parent authorized closing the shared delivery-metadata gap in the two
native symmetric-key insertion/build endpoints and their JNI bridges, with
no new ABI. Other argument errors and other native operations must preserve
their existing behavior.

## Red

```sh
CARGO_BUILD_JOBS=2 cargo test -p kmipkit-ffi --all-features --locked --test c_api_coverage public_c_fluent_builder_inserts_explicit_attributes_at_a_final_position -- --exact
bash bindings/java/gradlew -p bindings/java --offline --console=plain --no-daemon '-Dorg.gradle.jvmargs=-Xmx2g -XX:MaxMetaspaceSize=768m' test -x buildNativeLibrary --tests org.kmipkit.fluent.FluentFacadeShapeTest
```

C: exit 101; one failed, 55 filtered out. The compiled C consumer returns 2
when the existing out-of-range build error still reports delivery `NONE`.
The consumer also checks duplicate positions, rejected insertion, null
output, preserved typed attribute mismatch codes, and a subsequent unrelated
generic ABI error retaining delivery `NONE`.

Java: exit 1; nine tests, four failed because build errors are
`InvalidInputException` and catalogued-type insertion errors are
`TypedAttributeValueTypeMismatchException`, whereas the required
`ClientOperationException` must expose `NOT_SENT` and the original native
status/category/cause. Tests also require no connection to a real listening
socket and preserve argument/non-builder exception behavior.

The first Java compile attempt exhausted the default 512 MiB daemon heap
before executing tests. Repeating with the documented 2 GiB JVM setting ran
the expected Red tests. The generated `java_pid10703.hprof` was deleted.
For Red only, the baseline 0095 JNI binary was copied into the local ignored
Java target directory and `buildNativeLibrary` was skipped. Green rebuilds
the current JNI/Rust sources through the repository script.

The repository `target` is temporarily linked to the prior stack's Cargo
target cache to avoid duplicating build outputs; Cargo recompiles this
worktree's source. This symlink will be removed after verification without
removing the shared target directory.
