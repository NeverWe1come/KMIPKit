# ADR-0017: KMIP 2.1 Multi-Encoding Scope for 1.0

**Status**: Accepted by maintainer instruction on 2026-11-05
**Date**: 2026-11-05
**Decision owner**: KMIPKit maintainer
**Supersedes**: Encoding scope of [ADR-0002](0002-kmip-21-release-scope.md)
and raw-TLS response framing clauses in ADR-0005 and ADR-0015
**Specification**: [KMIPKIT-0054](../../specs/054-multi-encoding-governance/spec.md)

## Context

The original 1.0 boundary selected TTLV only and scheduled JSON/XML for later.
The maintainer has approved adding the OASIS KMIP 2.1 JSON and XML encodings
to 1.0. Their encodings are specified by KMIP Profiles v2.1 §§5.5 and 5.4.
The HTTPS client and server clauses in Profiles §§5.3.1–5.3.2 prescribe
Content-Type values for JSON and XML, then state that the body is binary TTLV.
Those statements conflict when read as unconditional requirements for one
message.

## Decision

1. The 1.0 encoding set is TTLV, JSON, and XML. TTLV is the default. The
   caller may select a different encoding for an individual exchange; the
   response must use that exchange's selected encoding.
2. Each encoding is supported over raw TLS and HTTPS/HTTP 1.1. Raw TLS carries
   one complete KMIP document per connection and closes after one response.
   JSON/XML raw TLS document completion is determined incrementally by the
   parser under the configured byte, depth, and element bounds; KMIPKit adds
   no proprietary length prefix.
3. The common internal substrate is the existing ordered `kmipkit-ttlv`
   `Item` model and checked-in catalog metadata. JSON/XML codecs are private
   implementation modules until KMIPKIT-0057 adds the client selector to the
   reviewed manifest and four language surfaces.
4. For HTTPS, KMIPKit interprets Profiles §§5.3.1–5.3.2 contextually: the
   binary TTLV body requirement applies when TTLV is selected. When XML or
   JSON is selected, send the corresponding `text/xml` or `application/json`
   body, with the matching Content-Type. This is KMIPKit's documented
   interpretation of an internal source inconsistency, not an OASIS erratum
   or an OASIS resolution. Do not claim formal HTTPS or JSON/XML profile
   conformance until all applicable clauses and official cases pass.
5. No automatic format negotiation, format fallback, or request retry is
   permitted. A response using another encoding is a payload-free protocol
   error with the established request-delivery state.
6. `Maximum Response Size` keeps its OASIS meaning computed as if the
   response were TTLV encoded. The local transport response-byte cap applies
   to actual received bytes for all encodings.

## Consequences

ADR-0005 and ADR-0015 continue to govern connection lifecycle, TLS, deadlines,
and no-retry behavior. Their raw-TTLV response-frame wording is superseded by
the one-complete-document rule above for JSON/XML.

The scope change affects the constitution, project definition, agent rules,
roadmap, API manifest, transport and client specs, generated bindings, tests,
and release evidence. The shared semantic model lets JSON and XML codecs be
implemented and tested in parallel after KMIPKIT-0054 stabilizes. The public
API manifest remains untouched by KMIPKIT-0054, KMIPKIT-0055, and
KMIPKIT-0056; KMIPKIT-0057 owns the coordinated public API change.

## Alternatives considered

- Keep JSON and XML after 1.0: rejected by the maintainer's approved scope
  change.
- Restrict JSON/XML to HTTPS: rejected; the maintainer selected both
  transports.
- Add a private framing prefix to raw TLS: rejected because it would create
  wire behavior not defined by the selected KMIP encoding profiles.
- Read the HTTPS TTLV body clause without contextual interpretation: rejected
  because it conflicts with the same client/server clauses' JSON/XML
  Content-Type conditions. The chosen interpretation is recorded for review
  and is not presented as official OASIS guidance.
