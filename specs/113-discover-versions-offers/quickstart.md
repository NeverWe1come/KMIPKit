# Quickstart: Validate Discover Versions Offers

## Prerequisites

- The feature implementation has been approved and built from one exact
  KMIPKit commit.
- Rust, C, Java 17, and Python 3.12 package test environments are available.
- A controlled mutual-TLS peer can capture one request and return a selected
  Discover Versions response.

## Scenarios

1. **Default compatibility**: Build Discover Versions with the existing
   no-argument API. Confirm the header uses the selected client protocol
   version and the payload offers only `(2,1)`.
2. **KMIP 2.0 and 2.1 offers**: Build an explicit list `[(2,1),(2,0)]` in
   each language. Confirm the normalized TTLV trees match exactly and the
   controlled peer observes the same field order.
3. **KMIP 2.0 only**: Offer `[(2,0)]` while independently configuring a 2.0
   header. Confirm both header and payload are 2.0, then repeat with a 2.1
   header and confirm that choosing offers does not rewrite it.
4. **Omitted offers**: Use the explicit no-offers builder. Confirm no Protocol
   Version child exists and that multiple well-formed response versions are
   retained in server order.
5. **Validation**: Confirm explicit empty and uncatalogued offer lists fail
   before transport I/O. Confirm malformed responses fail, and an unoffered
   response value is rejected only when an explicit list was sent.

## Expected Outcome

All languages produce the same normalized KMIP tree for equivalent inputs.
For deterministic TTLV values, encoded bytes are identical. No scenario
silently changes the configured header version or sends an automatic retry.
