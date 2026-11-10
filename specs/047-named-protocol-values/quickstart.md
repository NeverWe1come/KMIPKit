# KMIPKIT-0047 Quickstart Validation

1. Run the catalog-backed generator in check mode and confirm it reports all
   four language outputs current.
2. Compile a Rust consumer using one named Enumeration constant and one named
   Bit Mask value; pass an unknown raw Enumeration value through `from_raw`
   where a typed wrapper exists.
3. Compile and execute C, Java, and Python consumers using the corresponding
   constants and lookups.
4. Confirm a mixed known/unknown mask reports its known bits while retaining
   the original raw mask.
5. Verify generator check mode detects a deliberately stale temporary output
   fixture and never writes files.

The generated constants describe assigned values. They do not attest that a
server supports the value or that a cryptographic choice is appropriate.
