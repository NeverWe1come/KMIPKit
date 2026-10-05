# T014 Fixtures Independent Review

**Verdict: PASS**

## Scope

Reviewed the stable T014 fixture commit `64e17379c7bba621a44c7c5e649c1a9f2cd510d5` against its first parent `60cc941d28a1f3792aec643657f4fab67072f4a9`. Scope was the fixture test module and its `.hex` files only. This was a static review; no tests or other checks were run. No implementation files or task status were changed.

## Review results

- The valid fixture set covers all eleven TTLV Item Types: Structure, Integer, Long Integer, Big Integer, Enumeration, Boolean, Text String, Byte String, Date-Time, Interval, and Date-Time Extended.
- Representative bytes match the wire layout in the local OASIS KMIP 2.1 source. For example, the Integer fixture has a three-octet Tag, one-octet Type, four-octet Item Length, four value octets, and four padding octets; the Structure fixture's declared value extent contains its complete child; and the Big Integer fixture uses an eight-octet, sign-preserving representation.
- Valid and malformed normative fixtures state the OASIS document, relevant section, and stable NR attribution. Each fixture's attribution is checked against the corresponding case entry by the test module.
- The project-only vectors distinguish KMIPKit policy from OASIS requirements: empty Big Integer rejection is attributed to FR-006 while noting OASIS does not specify the nonempty minimum; trailing bytes are attributed to FR-004; and rejection of the catalogued reserved tag is attributed to FR-010/ADR-0011. OASIS references on the reserved-tag fixture establish the tag allocation context, not a decoder rejection requirement.
- Fixtures are bounded by the test helper, parsed as byte pairs, and assertions check either the expected decoded Item Type or the expected rejection category. The Structure case documents generic structural validation and does not claim schema validation.

## Findings and limits

No findings. This review confirms fixture content, coverage, and attribution by inspection; it does not establish that the tests compile or pass. The review is limited to the specified T014 fixture commit and does not assess later fuzzing or implementation changes. The T014 task's status remains under the coordinator's control.
