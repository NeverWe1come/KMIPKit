# Implementation plan

1. Replace regex parsing and the mock driver with strict configuration,
   versioned catalog loading, and shared result classification.
2. Build and install the JNI JAR from a clean tree at the exact release commit
   into an isolated Maven repository; verify artifact provenance.
3. Implement `plan`, `preflight`, and `run` with dedicated-environment,
   mutation, and environment-credential safeguards.
4. Use the public typed Java client and generic TTLV APIs; implement safe
   recipes for supported cases and precise reasons for unavailable routes.
5. Produce JSON, Markdown, and HTML reports conforming to the shared contract,
   and add a controlled TLS peer that checks typed requests and responses.
6. Run unit, schema, installed-package, and CI checks. Keep the PR in draft
   until implementation and required coverage are complete.
