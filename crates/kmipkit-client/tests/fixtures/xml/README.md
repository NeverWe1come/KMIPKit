# XML encoding fixture

`MSGENC-XML-M-1-21.xml` is the byte-identical OASIS KMIP Profiles v2.1
mandatory case linked by §5.4.4.1, acquired on 2026-11-05 from
<https://docs.oasis-open.org/kmip/kmip-profiles/v2.1/os/test-cases/kmip-v2.1/mandatory/MSGENC-XML-M-1-21.xml>.

SHA-256: `5ca1a4533bb8e911231ea9e43c446894ff11f9004e2ca74531515327145e75d3`.
The original copyright, source and TC IPR notices remain intact. Tests bind
`$NOW` to `2023-11-14T22:13:20Z` and replay each of the four message documents
in the fixture wrapper. This proves codec replay, not actual server behavior,
full client profile conformance, or certification. No build downloads it.
