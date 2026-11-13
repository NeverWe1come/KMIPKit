# KMIPKIT-0086 Implementation Evidence

## Exact release artifact pin

The Java consumer now installs KMIPKit from the exact `release/1.0.0` commit
`9a16cc90a12205101d22a38501a5e74c665df4dc`, matching the shared route
manifest. The Linux JNI JAR digest from the verified local build is
`65832c03d6c1cac569994ca3506f162335ccec409841cc3c43f6ccb1b8984944`.

Red: `ArtifactProvenanceTest.libraryPinMatchesTheExactSharedReleaseArtifactCommit`
failed while `library-pin.json` still contained the earlier release ancestor
`9692057c9ade474fb0c4e0b7130539da368eb794`.

Green: the test passes after aligning `library-pin.json` and the controlled TLS
test provenance check with the shared exact commit. The artifact was rebuilt by
`tools/install_artifact.py` from the pinned release tree; it was not reused from
the checkout build.

Verification run in the Java consumer project:

- `python3 -B -m unittest discover -s tests -p 'test_install_artifact.py' -v` — 3 passed.
- `bash ../../bindings/java/gradlew --no-daemon -PkmipkitMavenRepo="$PWD/build/maven-repository" test --tests org.kmipkit.validator.ArtifactProvenanceTest` — passed.
- `TMPDIR=/var/tmp python3 tools/install_artifact.py --repository ../.. --pin-file library-pin.json --maven-repository build/maven-repository --manifest build/artifact-manifest.json` — passed; source commit and JAR SHA-256 recorded above.

The Java consumer suite has been aligned with the common 2,689-case catalog and
expanded normative inventory. Broader shared-contract checks and PR CI remain
to be verified.

## Common catalog integration and KMIP 2.0

Red: after updating the expected catalog size and route contract, the Java
inventory tests failed because it treated local normative names such as
`Adjustment Type` as server operations. The run tests also showed that unknown
local TTLV targets were reported as library failures, and the old plan denied
all KMIP 2.0 typed routes.

Green: route inventory now classifies only `server_exchange` cases. Local
normative targets with no Java recipe remain `not_run` with an explicit reason,
so the report does not claim a library failure for untested targets. The
blanket KMIP 2.0 block is removed. A controlled TLS 1.3 mutual-authenticated
KMIP 2.0 Query run verifies that the request carries version 2.0 and that the
typed response succeeds. KMIP 2.0 Discover Versions remains pending because
the pinned library response parser uses a default supported-version offer of
KMIP 2.1 only.

Verification:

- `bash ../../bindings/java/gradlew --no-daemon -PkmipkitMavenRepo="$PWD/build/maven-repository" test` — 28 tests passed.
- `MainContractTest` confirms unsupported local target recipes remain pending and both protocol versions are represented by implemented routes.
- `ControlledTlsPeerTest.runSendsQueryWithConfiguredKmip20Version` passes against the installed `9a16cc90a12205101d22a38501a5e74c665df4dc` artifact.
