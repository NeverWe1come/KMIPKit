# KMIPKIT-0051 T002: Request Payload conditions audit

Date: 2026-11-05. Read-only source audit; no production changes, commits, or upstream edits.

## Scope and evidence

Primary source: pinned `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` ([kmip-spec-v2.1], OASIS Standard). SHA-256: `8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf`.

The inventory in `specs/051-pre-send-validation/gap-matrix.md` was compared field by field with the source HTML: **189/189 field names, REQUIRED cells, and one-based table ordinals match**, across 54 nonempty supported Request Payload tables. Tables 229 (Get Constraints) and 271 (Ping) are empty. Table 331 (Set Endpoint Role) is excluded by approved product scope. No order was inferred from numeric tag values.

Locators below identify the exact section/table/field row; `Lnnn` additionally means the beginning line in the pinned HTML. Paragraphs without numbers are located by opening text and source line. Short IDs R001–R189 mean `KMIPKIT-0051-R001`–`KMIPKIT-0051-R189`, preserving the matrix IDs.

The task concerns direct operation payload conditions and linked cross-field requirements. Detailed arbitrary attribute/value schemas remain governed by their defining §4 tables; this audit does not claim to enumerate every nested attribute semantic or profile clause. The per-row appendix records every inventory row explicitly, with applicable condition groups.

Locality:

- **L**: decidable from this request/schema without remote knowledge.
- **S**: depends on server object state, stored attributes, authorization, capability, execution history, or policy. Preserve supplied values; do not reject solely on this unknown.
- **A**: the source interaction has more than one plausible reading or lacks enough precision for a local decision. Apply the explicit project ruling recorded in `model-audit.md` and `traceability.md`; do not reject on the ambiguous point alone. Server-dependent truth remains with the server.
- **P**: existing approved project boundary, distinct from an assertion that OASIS mandates a particular interpretation.

## Shared schema rules

**BASE (all R001–R189, L):** enforce field types from their defining schema, the table's field order, and requiredness/cardinality in that exact operation context. §8 opening paragraph, L42440: all message fields SHALL appear in the specified order. Each appendix row cites its own Request Payload table and ordinal. `Yes` requires one occurrence, `No` permits absence, and `MAY be repeated` permits repetition; no universal singleton rule applies. `No for multi-part` occurs in the **REQUIRED** column and means non-required, never forbidden. Unknown/future data preservation remains the approved 0051 contract. Empty tables 229/271 define no standard request fields; no field IDs are invented for them.

**ID (R001, R002, R004, R008, R010, R015, R036, R049, R057, R059, R067, R072, R077, R079, R080, R110, R116, R123, R126, R137, R142, R145, R152, R156, R168, R172, R179; L/S):** the respective payload-table Unique Identifier row permits omission and specifies server ID Placeholder substitution (R010 only when no CSR is present). Do not locally require these identifiers or resolve/validate the placeholder's current server value. §6.1 paragraphs “The key management server SHALL implement…” (L22389) and “The ID Placeholder is obtained…” (L22397) specify batch lifetime and production/substitution, including Batch Order Option true or unspecified. Existence of a usable placeholder after prior batch operations is not proven by request field syntax. R029 is different: the optional identifier selects an existing key to split, not an implicit mandatory identifier. R087 and R053 are mandatory explicit request identifiers.

**ATTR (R013, R020, R022–R024, R034, R056, R091, R098, R104, R139, R150, R154, R158–R160; L/S):** §5.1–§5.4 Tables 157–160 (L21749–L22063) allow zero or more attribute members; “Attributes required” means the container is required, not generally that it must be nonempty. §4 opening and Table 27 attribute characteristics (L9594 onward) distinguish single-instance, multiple-instance, initial-set, modifiable and deletable properties. Enforce a known unambiguous client prohibition only when the operation context and attribute identity make it applicable; do not apply per-object server restrictions without the object information. Import overrides the initial-set rules (IMPORT below). Locate has special partial-structure/range semantics (LOCATE below). Server defaults are explicitly described by §4 paragraph L9747/Table 28 (L9771–L9856) and §6.1.53 (L35979), so lack of arbitrary factory attributes must not become an invented universal rejection. Derive Key has explicit stronger output requirements (DERIVE below).

**ATTRIBUTE-OPS (R003, R005–R007, R050–R051, R124–R125, R169; L/S/A):** New/Current Attribute contain a single actual attribute member (§5.6 Table 162, L22210–L22284; §5.7 Table 163, L22287–L22361). Attribute Reference is the §5.5 Table 161 alternative: Enumeration(Tag), or Structure containing required Vendor Identification and Attribute Name; the vendor identifier's allowed alphabet is [A-Za-z0-9_.] (L22117). Add Attribute must not change an existing value or add read-only attributes (§6.1.2 opening, L22679); Adjust and Set must not add/modify read-only attributes (§6.1.3 L22870; §6.1.51 L35629). Known read-only restrictions can be local; whether a value exists or an attribute has one/multiple stored instances is S. Delete never removes attributes that must always have a value (§6.1.13 L25621), but applicability may require the remote object type. Modify permits only an existing instance and uses implicit sole-instance selection if Current Attribute is absent (§6.1.34 L30605): that existence/count is S. Do not infer that optional Current Attribute must be present.

**ATTRIBUTE-WRAPPER (R283; L):** Typed Current/New Attribute wrappers carry one direct attribute Item, but callers can wrap a raw generic Item that bypassed `AttributeSet::try_new`. For Vendor Attribute (`0x420008`), validate the selected §4.60 Table 150 shape and required-member order/types before an attribute-mutation request is sent. Other remote-object facts and the meaning of arbitrary Attribute Value content remain outside this check.

**ATTRIBUTE-TYPE (R284; L):** When a raw Current/New Attribute Item uses a catalogued standard attribute tag, enforce that attribute's exact permitted TTLV Item Type from its pinned §4 definition. Multiple allowed forms (such as Unique Identifier) remain accepted. Unknown assigned, extension, and vendor tags have no catalogued type assertion here and remain generic.

**ADJUST (R005–R007, L/S/A):** §11.1 Table 428, Increment/Decrement rows L45880/L45896, applies to Integer, Long Integer, Big Integer, Interval, Date-Time and Date-Time Extended and supplies a default adjustment of 1 (seconds/microseconds for dates); optional Adjustment Value is therefore valid. Negate applies to Integer, Long Integer, Big Integer and Boolean (L45912). §6.1.3 L22870 defines zero/false initialization and failure for other missing remote values. Target type can be checked only where known from a standard Attribute Reference; an unknown/vendor attribute is not enough information. Table 428 uses the wording “Adjustment Parameter” while Table 170 names the field “Adjustment Value”; it does not explicitly forbid an unused Adjustment Value with Negate, mandate the adjustment value's TTLV type for every target type, or define a universal positive-only constraint.

## Operation and cross-field condition groups

**CERT (R010–R013, L/S/A):** §6.1.6 Table 179 Certificate Request Type row L23524 explicitly requires Type when Certificate Request Value is present. That is a one-way implication. The source does not state the reverse implication, an XOR between explicit identifier and CSR, or that the identifier must be explicitly supplied rather than omitted for server placeholder resolution. The table's Unique Identifier row L23503 specifically provides the omitted-identifier/no-CSR case. §6.1.6 paragraph “The Certificate Request object MAY be omitted…” L23446 requires Certificate Type in Attributes **when both Request Type and Request Value are omitted**. That nested attribute requirement is L. Paragraph L23469 gives CSR information precedence over Attributes if they conflict, rather than rejecting conflicting inputs. Type-only requests and the meaning of “Certificate Request object” versus the table's “Certificate Request Value” terminology deserve A, not an invented reverse dependency. Only one certificate is requested (L23440), structurally supported by singleton fields.

**CHECK (R015–R018, L/S):** §6.1.7 paragraphs L23712 and L23731 recommend Check within a batch, Batch Order Option true, and STOP or UNDO continuation. These are SHOULDs, not unconditional field-requiredness rules; their verification needs batch context. §6.1.7 Table 182 (L23752–L23812) says Usage Limits Count, Cryptographic Usage Mask and Lease Time ask about server policy/available resources. Do not compare their values to unknown stored limits or reject an intentionally different Locate mask. Paragraph L23815 explicitly says these are not encoded in an Attribute structure.

**CREATE (R019–R020, L/S):** §6.1.8 opening L24098 limits the created managed object to a new Symmetric Key or Secret Data. It does not list universal Cryptographic Algorithm/Length input requirements for all factory requests. Attributes container remains mandatory; server algorithm/capability and defaults are S.

**KEYPAIR (R022–R024, L/S):** §6.1.9 L24339 gives individual Private/Public Key Attributes precedence over Common Attributes. The paragraph directly following Table 189, L24529, requires union for multi-instance attributes. Supplying common plus key-specific attributes is permitted, not an exclusion; do not flag duplicates across those different containers merely because one overrides/merges the other. No request Attributes container is mandatory in Table 189.

**SPLIT (R028–R035, L/S/A):** §6.1.10 Table 193 requires Parts, Threshold, Method and Attributes; optional existing-key identifier selects split-existing versus server generation (L24760), and the existing key's attributes win conflicts. §2.8 Table 9 Prime Field Size row L7443 conditionally requires that field for Polynomial Sharing Prime Field; the wording is conditional requiredness, not an explicit “SHALL NOT otherwise”. §13.1 XOR bullet L65940 expressly says Parts equals Threshold; this equality is L. The Prime Field bullet L65947 requires a prime greater than 2^L, where L is the bit length of the secret (L65937): method/field presence is L; the size comparison may be S because the secret/key is server-side. §2.8 L7317 explicitly bounds **Key Part Identifier** to [1, Parts] in an actual Split Key object, not a Create Split Key request field. “Threshold is the minimum number of parts needed” supports semantic range discussion but neither Table 193 nor that paragraph spells out a numeric request bound; do not present inferred Parts/Threshold positivity or Threshold<=Parts as a quoted SHALL. Table 193 has exactly one Attributes container, not one per split.

**STREAM (R038, R040–R042, R061, R063–R065, R083–R086, R112–R115, R118–R122, R174–R178, R181–R186; L/S/A/P):** §6.1 multipart procedure, L22417–L22498, and §7.8 L39126/Table 359, §7.14 L39744/Table 366, §7.17 L39865/Table 369 provide initialization, continuation, finalization, and supplied Correlation Value structure. First multipart request supplies Init=true; subsequent requests return the server-issued Correlation Value; final supplies Final=true. Actual correlation provenance and multipart capability are S. A request alone cannot distinguish an intended continuation with all framing accidentally omitted from an unframed single-part request; do not invent hidden state.

The exact Data sentence in §6.1 L22498–L22499 says Data is always required except when either Init or Final is true. Its “required” is lowercase, not uppercase RFC REQUIRED. It establishes the existing intermediate-part Data rule for Encrypt/Decrypt/Hash/MAC/Sign but must be reconciled with the operation-specific alternatives below. REQUIRED=No for multi-part does **not** prohibit Data and is narrower than saying all multipart calls may omit it. The matrix's initial inventory descriptions saying “omitted for multi-part” should be corrected to “not required by the table for multi-part; see §6.1 and open discrepancy”.

| Data-like field | IDs and table evidence | Scope of general Data paragraph |
|---|---|---|
| Data in Encrypt/Decrypt/Hash/MAC | R061 Table 214 L26558; R038 Table 196 L25132; R083 Table 235 L28270; R112 Table 259 L30060 | Literal Data fields. Required for unframed single-part by respective table; intermediate-part Data required by existing §6.1 contract; first/final parts may omit under §6.1. DISC-045 protects framed single-part without Data. |
| Data in Sign | R174 Table 334 L36391 | Table requires single-part Data **unless Digested Data is supplied**. Do not reject that explicit alternative by blindly applying a global Data-only rule. DISC-045 framed-single-part conflict applies when both Data and Digested Data are absent. Digest-only unframed single-part versus general paragraph is a separate unadjudicated source question. |
| Original Data in MAC Verify | R118 Table 262 L30371; §6.1.33 paragraph L30281 | Optional for algorithms that need it; do not make it globally mandatory or add this operation to DISC-045. |
| MAC Data in MAC Verify | R119 Table 262 L30387 | Distinct field; required single-part, table non-required multi-part. §6.1's literal Data paragraph does not itself require MAC Data on every intermediate request. |
| Original Data in Signature Verify | R181 Table 337 L36726; §6.1.56 paragraph L36630 | Optional for algorithms that need it; do not make it globally mandatory or add this operation to DISC-045. |
| Digested Data in Sign/Signature Verify | R175 Table 334 L36407; R182 Table 337 L36742 | Distinct optional field. No explicit Data/Digested Data XOR is stated in these rows or operation prose. |
| Signature Data in Signature Verify | R183 Table 337 L36758 | Distinct field; required single-part, table non-required multi-part. The general Data sentence does not itself mandate intermediate Signature Data. |

**DISC-045 boundary (P/A):** retain the approved existing gate for a framed single-part request with Init=true, Final=true, **no Correlation Value**, and omitted Data: Encrypt, Decrypt, Hash, MAC, and Sign (Sign conflict when Digested Data is also absent). Accept the corresponding Data-present shape. Do not silently close the discrepancy or broaden its operation set to MAC Verify or Signature Verify. This is the existing project's conservative support boundary, not a conclusion that the conflicting source texts have been reconciled. Existing project evidence: `specs/019-cryptographic-operations/spec.md` lines 97, 115, 180; `specs/021-hash-mac-signature-operations/spec.md` lines 25–27; `specs/010-profile-conformance/research.md` line 494. This audit does not modify those documents or choose between the sources.

**CRYPTO-CONTEXT (R037, R039, R060, R062, R111, R117, R173, R180; L/S):** respective table optional Cryptographic Parameters rows allow the object-associated parameters to supply them and specify server failure if absent and needed; these are not a local mandate to supply parameters universally. Encrypt §6.1.17 paragraph L26466 allows IV omission if server Random IV is requested or the algorithm needs no IV; Decrypt §6.1.11 L25049 allows omission if no IV is used. Whether the managed object algorithm/attributes require these fields is often S. Reuse approved 0050 §4.16 rules rather than adding algorithm-policy guesses.

**AEAD (R043–R044, R066; L/S):** Table 196 Additional Data row L25214 and Tag row L25232 explicitly require each supplied field to be on the **initial** multipart Decrypt request. Table 214 Additional Data row L26639 gives the same initial-only rule for Encrypt. These are L when framing identifies a subsequent part. No universal requirement to include Additional Data or Tag is stated by these optional rows, and no algorithm/size/authentication decision is possible from framing alone. The encrypt returned tag timing is response behavior (Table 215 L26762), not an input rule.

**DERIVE (R052–R056; L/S/A):** §6.1.14 opening L25832 permits Symmetric Key or Secret Data output, requires requested Cryptographic Length for **all methods**, and also Cryptographic Algorithm for a key. Table 205 Attributes row L25980 explicitly requires length and algorithm for Symmetric Key. Table 205 identifier row L25919 requires one or more explicit Unique Identifiers and forbids substituting the current ID Placeholder; do not impose an invented upper count or uniqueness rule. The input objects' Derive Key usage bit, object types, and derivation output limit are S.

Nested §7.12 Table 364 (L39384–L39514): Derivation Data may repeat and is required unless a Secret Data identifier supplies it implicitly; Salt and Iteration Count are required for PBKDF2; Initialization Vector may be omitted with empty-IV semantics; Cryptographic Parameters conditional on PRF. Paragraph L39516 explicitly requires Hashing Algorithm for HASH and Block Cipher Mode for the stated AES-CBC example; L39523 says HMAC ignores supplied parameters because its stored key attributes suffice. Paragraph L39531 forbids **both explicit Derivation Data and an identifier known to designate the implicit Secret Data input**. Merely having a Unique Identifier does not prove this mutual exclusion, because it may designate an ordinary derivation key; do not reject explicit derivation data plus every identifier. The same paragraph allows HASH to use a derivation key or derivation data; tension with Table 205's mandatory one-or-more identifier list is A. AWS Signature Version 4 paragraph L39538 orders Date, Region, Service as Derivation Data; it does not separately provide per-component tags/encodings. HKDF paragraph L39541 specifies managed-object Input Key Material, Salt, optional Derivation Data information, output length in Cryptographic Length, and default SHA-256 unless overridden; do not invent a universal client-selected hash requirement or unconditional HKDF Derivation Data requirement from the generic table.

**WRAP-RETURN (R068–R071, R073–R076; L/S):** §6.1.18 L26846 says these Export options have Get semantics. §6.1.19 L27135–L27149 describes format conversion/server capability and PKCS#12 links/certificate chain; remote conversion support, key type and chain correctness are S. §7.18 paragraph L39907 requires Key Wrapping Specification when the client requests wrapping. That structure's Table 370 L39987–L40068 requires Wrapping Method and at least one of Encryption Key Information or MAC/Signature Key Information; **both are allowed**, not XOR. Encoding Option omission selects TTLV encoding; No Encoding forbids attributes in the wrapped Key Value (L39941). Parameters must match stored wrapping-key parameters; the server comparison is S. No direct mandatory Key Wrap Type/Key Wrapping Specification pairing beyond the actual requested wrapping semantics is established by these rows.

**GET-ATTRS (R078; L/S):** §6.1.20 first paragraph L27440 explicitly prohibits repeating the same Attribute Reference within a request. Distinct references may repeat the field tag; zero references means get all (L27449). Duplicate known references are L, existence of requested attributes is S.

**IMPORT (R087–R092; L/S):** §6.1.25 Table 238 Key Wrap Type row L28558 says **if and only if the key object is wrapped**: required on wrapped key objects, absent on unwrapped key objects. Wrapped/plain is locally recognizable from a known §2/§3 object shape; an opaque future object requires A rather than an invented decision. Replace Existing row L28540 uses absent/false versus true only to govern server collision handling; existence of the target identifier is S. Opening paragraph L28466 overrides attribute “Initially set by”/“When implicitly set” rules: all supplied values are to be used. Do not reject Import just because an attribute is normally server-initialized.

**KEY-OBJECT (R092, R140; L/S/A):** Any Object means one actual selected §2 object, not a field tagged Object Type; check known Object Type/object shape correspondence when its type is known. §3.1 Table 12 (L7654, L7736–L7798) says wrapped Key Value is Byte String, plaintext Key Value is Structure, and Key Wrapping Data is present iff wrapped; Algorithm/Length have paired presence and may be omitted only when available from Key Value, with Secret Data/Opaque exception. §3.2 Table 13 (L7893–L7915) selects Key Material byte string versus transparent Structure by known Key Format Type. §3.3 paragraph L8064/Table 15 requires at least one wrapping-key-information structure, permits both, and defaults absent wrapping crypto parameters from the remote keys (S). No Encoding must not wrap attributes (L8041). Unknown future formats cannot be rejected solely for not being recognized.

**INTEROP (R093–R094; S):** §6.1.26 L28727 says Interop is unavailable in a production **server**. Client validation must not guess whether a contacted server is production or forbid the request based on that unknown.

**JOIN (R095–R099; L/S/A):** §6.1.27 opening L28899/Table 244 identifier row L28959 require identifier count at least each input Split Key's Threshold. The request itself contains no threshold, so beyond the table's explicit **one-or-more** cardinality the count is S unless actual input Split Key metadata is separately supplied. Paragraph L28904 says a client MAY include Secret Data Type when output is Secret Data. It does not explicitly say “SHALL NOT” for other outputs; no invented biconditional requiredness or automatic default is justified.

**LOCATE (R100–R104; L/S/A):** §6.1.28 L29150 makes Offset=0 equivalent to omission and permits both Offset and Maximum Items (skip offset, then return up to maximum). No positive-only Maximum/Offset bound or special Maximum=0 meaning is explicitly given in the cited payload prose/table. Storage Status Mask absence means online-only (Table 247 L29297); don't reject archived/destroyed bits or unknown preserved bits as a local policy. Attributes container is required but explicitly may be empty (L29337). Paragraph L29181 permits **two instances of the same Date attribute** for range matching; it gives no explicit rejection rule for more than two or explicit order of the endpoints. Paragraph L29206 permits partial fields in structured attribute filters, including Linked Object Identifier without Link Type; a normal full object-attribute schema must not reject this request filter. Mask containment, Usage Limits comparisons, group-fresh/default selection, and search limits are server matching/policy (L29191–L29220). Object Group Member does not explicitly force an Object Group filter to be present.

**LOGIN-RIGHTS (R045–R048, R106–R108; L/S/A):** Tables 199/253 explicitly describe Lease Time as **Interval or Date-Time**, unlike the managed-object Lease Time attribute (§4.30 Table 88 Interval). R046/R107 Request Count is Integer; no explicit positivity/minimum appears in these request rows. Table 199 requires Rights container but §7.35 Table 387 (L41414) allows zero or more Right structures, so no universal nonempty-Rights constraint. §7.34 L41241 says omitted Right aspects are unrestricted; Table 386 makes Usage Limits/Operations/Objects/Object Groups optional. §7.40 Table 392 requires Usage Limits Total, Count, Unit in that nested structure, but granting the ticket and enforcing permitted operations is S (§6.1.12 L25399).

**PKCS (R127–R130; L/S/A):** Table 273 L31184 supplies default V3.0 interface if omitted; Function row L31203 uses Enumeration for PKCS#11 functions or Integer for vendor functions. Correlation row L31222 says the server-provided correlation must be returned after a previous response; a stateless request does not establish that history. Input Parameters row L31241 defers format to the PKCS#11 Profile and external standard. This base-spec audit does not invent per-function parameter cardinalities or server capability tests.

**ASYNC (R009, R131–R132, R135–R136; L/S/A):** Cancel/Poll require one correlation value (Tables 176/276 and §6.1.5 L23262 / §6.1.38 L31460); Process Table 278 requires one. Existence/status belongs to server. Table 285 L32456/L32475 explicitly defines optional filter containers with **zero or more** members; §7.1 Table 352 and §7.25 Table 377 define their structures. Paragraph L32500 permits no filters, correlation filters, or operation filters but specifies no exclusion between both containers and no precise intersection/union rule when both are supplied. Preserve both; no XOR. The plural Asynchronous Correlation Values container's tag ambiguity remains the matrix's open tag question, separate from these known structural/cardinality facts.

**QUERY (R133–R134; L/S):** Table 282 and paragraph L31992 require one-or-more Query Function values. No deduplication prohibition appears here. Paragraph L31979 explicitly allows both Query Extension List and Query Extension Map: the server ignores List when Map is present. This is a valid combination, never a mutual exclusion. Object Groups is optional; no function-dependent presence mandate appears in the request table/prose.

**REVOKE (R143–R144; L/S):** Table 295 Compromise Occurrence Date row L33213: Date SHOULD be supplied for Key Compromise or CA Compromise; SHALL NOT be supplied for any other known Revocation Reason. Only the forbidden-known-reason case is an unconditional hard local failure. Missing date for compromise is a SHOULD, not REQUIRED; the server's fallback to Initial Date is in §6.1.44 paragraph L33132. An unknown future reason must not be rejected solely for being unknown.

**RECERT (R145–R151; L/S/A):** Table 300 Request Type row L33663 requires Type when Request Value present; no explicit reverse dependency or exclusion with Certificate Request Unique Identifier is given. §6.1.45 paragraph L33348 allows omitted CSR, falls back to existing certificate's Certificate Type when no new Type is supplied, and therefore does **not** inherit Certify's unconditional Certificate Type-in-Attributes rule. Paragraph L33362 gives CSR information precedence over attributes. Table 298 / paragraph L33377 offsets and Table 299 replacement attributes require existing certificate metadata, so are S; Offset is optional Interval (Table 300 L33700), not a date and not a local current-clock comparison. “Only once” in paragraph L33366 is SHOULD and depends on history.

**REKEY (R152–R163; L/S):** §6.1.46 paragraphs L33873–L33889, Tables 303–305, and §6.1.47 paragraphs L34419–L34439, Tables 308–310 describe replacement/copying/date rules from the existing object. Optional Offset, absent-offset date copying, name transfer, and replacement links are S; no local requirement to supply Offset or any optional attribute container follows. Re-key applies to existing Symmetric Key; Re-key Key Pair uses existing private-key identifier; these referenced object types are S unless their objects are locally known. A “SHOULD only once” historical rule is not a stateless request failure.

**REPROVISION (R164–R165; L/S/A):** §6.1.48 opening L35070 presents three alternatives: CSR, certificate, or no parameters (server creates credential). Tables mark each input optional; no-input request is valid. Supplying both is not described, and the prose's “or” supports an alternative reading, but no explicit “not both” or precedence rule is stated. Record dual-input interpretation for reviewer disposition rather than silently asserting a quoted SHALL NOT. Existing credential expiry/revocation/compromise and caller-identity checks (L35093–L35102) are S.

**RNG (R166–R167; L/S/A):** Table 316 Data Length and §7.10 Table 362 define Integer byte count, but no explicit numeric positivity/minimum is given there; do not fabricate >0 as an OASIS clause. Table 319 requires Data; generic §7.9 Tables 360–361 admits Byte String, Enumeration, Integer representations of Data, so no global “Data must always be Byte String” schema. §6.1.50 L35462 permits server refusal/zero data-used and says client must not treat ignored seed material as an error. RNG acceptance/quality is S.

**CONSTRAINTS-DEFAULTS (R170–R171; L/S):** Table 325 requires Constraints container; §7.7 Table 358 (L39100) permits zero or more Constraint structures. §7.6 Table 357 (L39006–L39038) makes Object Types/Object Groups/Attributes optional. Table 328 Defaults Information is optional and omitting it removes all defaults; §7.11 Table 363 allows zero or more Object Defaults. §7.22 Table 374 gives Object Type/Object Types **alternative** plus required Attributes and optional Object Groups; its merged alternative cell needs schema-aware parsing, not three independent fields. Whether constraints/defaults are authorized or supportable is S.

**VALIDATE (R187–R189; L/S/A):** §6.1.57 opening L36998 requires a single chain per request. Paragraph L37002 explicitly permits certificate objects and/or stored-certificate identifiers to compose it, so both representations together are valid. Table 340 permits repetition independently for Certificates and Unique Identifiers; order is certificate block, identifier block, optional Validity Date, not interleaving by call order. No explicit joint minimum count appears in the table; nonempty-chain enforcement is a semantic reading of “a certificate chain”, not a direct 1..* marker. Preserve the existing approved nonempty-chain model boundary if applicable and record the inference rather than claiming the table explicitly requires one. One-chain membership/content, validation method, trust anchors and validation order are S (L37008). Omitted Validity Date defaults to current time (Table 340 L37081); do not insert a client timestamp or require an explicit date.

**MASKS (R014, R021, R025–R027, R035, R099, R141, R151, R155, R161–R163; L/S/A):** §7.33 paragraphs L41163–L41187 define an **ordered collection of alternative** permissible masks, all bits of one mask must match, and common/private/public variants have identical layout. Whether the server can satisfy any selection is S, not a local capability failure. Table 385 (L41193–L41232) contains only the outer Structure row and **no child rows**. The prose establishes multiple selections and alternativeness but the table does not provide an explicit child minimum/cardinality. Do not fabricate nonempty/singleton child validation or reject alternative masks as inconsistent merely because they differ.

**VERSIONS (R058; L/S):** §6.1.16 paragraph L26285/Table 211 L26333 requires decreasing **client preference**, not decreasing numeric protocol version. The order of caller preference must be preserved; a validator cannot reconstruct preference from version numbers. Which versions the server supports is S.

**USAGE (R080–R081; L/S):** §6.1.23 L28005–L28021 requires protection-enabled objects with Usage Limits, sufficient remaining allocation, and permitted time window. These referenced-object facts are S. Table 232 L28080 requires Usage Limits Count, but does not state a positive-only numeric bound. §4.59 L20820 prohibits setting/modifying Usage Limits Count via Add/Modify and says creation ignores supplied Count; apply those known operation-specific rules rather than globally forbidding a count in Check/Allocation/Login nested structures.

**OBJECT-STATE (R001, R008, R057, R126, R137; S):** Activate requires Pre-Active (§6.1.1 L22513); Archive is only client permission and timing/storage is server policy (§6.1.4 L23095); Destroy requires Pre-Active or Deactivated (§6.1.15 L26118); Obtain Lease issuance depends on server policy and existing lease semantics (§6.1.35 L30819); Recover applies to archived object retrieval and may need async handling (§6.1.42 L32613). Do not turn object identifier syntax into a claim that this remote condition is known.

## Standards ambiguities requiring retained dispositions

1. **A-DATA / existing DISC-045:** table single-part requiredness versus §6.1 lowercase “required” exception for a framed single-part request. Preserve approved boundary; do not close or broaden it. Digest-only unframed Sign remains a separate unadjudicated question.
2. **A-CSR:** only Request Value -> Request Type is explicit. Reverse implication, type-only requests, unique identifier/CSR exclusivity, and Recertify CSR-ID/value coexistence are not settled by the cited text.
3. **A-DELETE:** Table 202 permits both selectors independently. §6.1.13 describes Current Attribute, or reference when Current absent, but supplies no explicit XOR, no both-absent error, and no both-present precedence. Do not manufacture those constraints.
4. **A-DERIVE:** Table 205 one-or-more identifiers versus §7.12 HASH key-or-data alternatives; generic Derivation Data requiredness versus HKDF optional information; whether an identifier refers to Secret Data is often remote knowledge.
5. **A-SPLIT:** no explicit hard positivity/range request clause beyond XOR equality and meaning of total/minimum. Conditional “required only if” Prime Field Size is not an independently stated prohibition in all other methods; comparing prime size to hidden key length is not local.
6. **A-REPROVISION:** three prose alternatives do not spell out handling of both optional fields together.
7. **A-ASYNC:** both filter containers are permitted but combination semantics (intersection/union) are unstated. The plural correlation container tag is resolved by §7.1 Table 352 and §11.56 Table 487 as 0x420176 (see `model-audit.md`).
8. **A-VALIDATE:** joint nonempty-chain minimum is implied by purpose, not explicit optional-row cardinality; distinguish this from unambiguously allowed mixtures.
9. **A-RANGES:** no explicit >0 rule found in the cited Request Count, RNG Data Length, Maximum Items, Offset Items, or Usage Limits Count rows. Natural count semantics are not a license to quote an absent SHALL.
10. **A-MASKS:** Table 385 omits member rows; ordered alternative masks are defined in prose, not an explicit child minimum/cardinality.
11. **A-ADJUST:** “Adjustment Parameter” naming/type/Negate unused-field handling is not precisely specified by Tables 170/428.

The source report deliberately identifies interpretation questions. Project ruling: error-severity checks are added only where OASIS states a requirement and the supplied request makes it decidable; an A-marked ambiguity does not by itself authorize a local rejection. The existing KMIPKIT-DISC-045 gate remains a separate project boundary, not a claim that the OASIS discrepancy is resolved.

## Complete per-field evidence ledger

Every row below has BASE. The conditional REQUIRED text is copied from the actual source cell. Additional groups above identify the precise local/server boundary. “Schema only” means no extra conditional rule was verified for that direct field beyond its defining nested schema; it does not assert that every possible nested value was audited.

| Rule | Operation / field | Order | Exact source row | REQUIRED (verbatim) | Additional condition groups |
|---|---|---:|---|---|---|
| R001 | Activate: Unique Identifier | 1 | §6.1.1, Table 164, Unique Identifier row (L22548) | No | ID, OBJECT-STATE |
| R002 | AddAttribute: Unique Identifier | 1 | §6.1.2, Table 167, Unique Identifier row (L22718) | No | ID |
| R003 | AddAttribute: New Attribute | 2 | §6.1.2, Table 167, New Attribute row (L22738) | Yes | ATTRIBUTE-OPS |
| R004 | AdjustAttribute: Unique Identifier | 1 | §6.1.3, Table 170, Unique Identifier row (L22909) | No | ID |
| R005 | AdjustAttribute: Attribute Reference | 2 | §6.1.3, Table 170, Attribute Reference row (L22929) | Yes | ATTRIBUTE-OPS, ADJUST |
| R006 | AdjustAttribute: Adjustment Type | 3 | §6.1.3, Table 170, Adjustment Type row (L22947) | Yes | ATTRIBUTE-OPS, ADJUST |
| R007 | AdjustAttribute: Adjustment Value | 4 | §6.1.3, Table 170, Adjustment Value row (L22965) | No | ATTRIBUTE-OPS, ADJUST |
| R008 | Archive: Unique Identifier | 1 | §6.1.4, Table 173, Unique Identifier row (L23133) | No | ID, OBJECT-STATE |
| R009 | Cancel: Asynchronous Correlation Value | 1 | §6.1.5, Table 176, Asynchronous Correlation Value row (L23297) | Yes | ASYNC |
| R010 | Certify: Unique Identifier | 1 | §6.1.6, Table 179, Unique Identifier row (L23503) | No | ID, CERT |
| R011 | Certify: Certificate Request Type | 2 | §6.1.6, Table 179, Certificate Request Type row (L23524) | No | CERT |
| R012 | Certify: Certificate Request Value | 3 | §6.1.6, Table 179, Certificate Request Value row (L23543) | No | CERT |
| R013 | Certify: Attributes | 4 | §6.1.6, Table 179, Attributes row (L23561) | No | ATTR, CERT |
| R014 | Certify: Protection Storage Masks | 5 | §6.1.6, Table 179, Protection Storage Masks row (L23581) | No | MASKS |
| R015 | Check: Unique Identifier | 1 | §6.1.7, Table 183, Unique Identifier row (L23848) | No | ID, CHECK |
| R016 | Check: Usage Limits Count | 2 | §6.1.7, Table 183, Usage Limits Count row (L23867) | No | CHECK |
| R017 | Check: Cryptographic Usage Mask | 3 | §6.1.7, Table 183, Cryptographic Usage Mask row (L23887) | No | CHECK |
| R018 | Check: Lease Time | 4 | §6.1.7, Table 183, Lease Time row (L23906) | No | CHECK |
| R019 | Create: Object Type | 1 | §6.1.8, Table 186, Object Type row (L24139) | Yes | CREATE |
| R020 | Create: Attributes | 2 | §6.1.8, Table 186, Attributes row (L24157) | Yes | ATTR, CREATE |
| R021 | Create: Protection Storage Masks | 3 | §6.1.8, Table 186, Protection Storage Masks row (L24178) | No | MASKS |
| R022 | CreateKeyPair: Common Attributes | 1 | §6.1.9, Table 189, Common Attributes row (L24384) | No | ATTR, KEYPAIR |
| R023 | CreateKeyPair: Private Key Attributes | 2 | §6.1.9, Table 189, Private Key Attributes row (L24406) | No | ATTR, KEYPAIR |
| R024 | CreateKeyPair: Public Key Attributes | 3 | §6.1.9, Table 189, Public Key Attributes row (L24426) | No | ATTR, KEYPAIR |
| R025 | CreateKeyPair: Common Protection Storage Masks | 4 | §6.1.9, Table 189, Common Protection Storage Masks row (L24446) | No | MASKS |
| R026 | CreateKeyPair: Private Protection Storage Masks | 5 | §6.1.9, Table 189, Private Protection Storage Masks row (L24471) | No | MASKS |
| R027 | CreateKeyPair: Public Protection Storage Masks | 6 | §6.1.9, Table 189, Public Protection Storage Masks row (L24496) | No | MASKS |
| R028 | CreateSplitKey: Object Type | 1 | §6.1.10, Table 193, Object Type row (L24801) | Yes | SPLIT |
| R029 | CreateSplitKey: Unique Identifier | 2 | §6.1.10, Table 193, Unique Identifier row (L24817) | No | SPLIT |
| R030 | CreateSplitKey: Split Key Parts | 3 | §6.1.10, Table 193, Split Key Parts row (L24834) | Yes | SPLIT |
| R031 | CreateSplitKey: Split Key Threshold | 4 | §6.1.10, Table 193, Split Key Threshold row (L24850) | Yes | SPLIT |
| R032 | CreateSplitKey: Split Key Method | 5 | §6.1.10, Table 193, Split Key Method row (L24867) | Yes | SPLIT |
| R033 | CreateSplitKey: Prime Field Size | 6 | §6.1.10, Table 193, Prime Field Size row (L24882) | No | SPLIT |
| R034 | CreateSplitKey: Attributes | 7 | §6.1.10, Table 193, Attributes row (L24897) | Yes | ATTR, SPLIT |
| R035 | CreateSplitKey: Protection Storage Masks | 8 | §6.1.10, Table 193, Protection Storage Masks row (L24913) | No | SPLIT, MASKS |
| R036 | Decrypt: Unique Identifier | 1 | §6.1.11, Table 196, Unique Identifier row (L25093) | No | ID |
| R037 | Decrypt: Cryptographic Parameters | 2 | §6.1.11, Table 196, Cryptographic Parameters row (L25111) | No | CRYPTO-CONTEXT |
| R038 | Decrypt: Data | 3 | §6.1.11, Table 196, Data row (L25132) | Yes for single-part. No for multi-part. | STREAM |
| R039 | Decrypt: IV/Counter/Nonce | 4 | §6.1.11, Table 196, IV/Counter/Nonce row (L25149) | No | CRYPTO-CONTEXT |
| R040 | Decrypt: Correlation Value | 5 | §6.1.11, Table 196, Correlation Value row (L25165) | No | STREAM |
| R041 | Decrypt: Init Indicator | 6 | §6.1.11, Table 196, Init Indicator row (L25182) | No | STREAM |
| R042 | Decrypt: Final Indicator | 7 | §6.1.11, Table 196, Final Indicator row (L25198) | No | STREAM |
| R043 | Decrypt: Authenticated Encryption Additional Data | 8 | §6.1.11, Table 196, Authenticated Encryption Additional Data row (L25214) | No | AEAD |
| R044 | Decrypt: Authenticated Encryption Tag | 9 | §6.1.11, Table 196, Authenticated Encryption Tag row (L25232) | No | AEAD |
| R045 | DelegatedLogin: Lease Time | 1 | §6.1.12, Table 199, Lease Time row (L25434) | No | LOGIN-RIGHTS |
| R046 | DelegatedLogin: Request Count | 2 | §6.1.12, Table 199, Request Count row (L25452) | No | LOGIN-RIGHTS |
| R047 | DelegatedLogin: Usage Limits | 3 | §6.1.12, Table 199, Usage Limits row (L25471) | No | LOGIN-RIGHTS |
| R048 | DelegatedLogin: Rights | 4 | §6.1.12, Table 199, Rights row (L25489) | Yes | LOGIN-RIGHTS |
| R049 | DeleteAttribute: Unique Identifier | 1 | §6.1.13, Table 202, Unique Identifier row (L25661) | No | ID |
| R050 | DeleteAttribute: Current Attribute | 2 | §6.1.13, Table 202, Current Attribute row (L25681) | No | ATTRIBUTE-OPS |
| R051 | DeleteAttribute: Attribute Reference | 3 | §6.1.13, Table 202, Attribute Reference row (L25700) | No | ATTRIBUTE-OPS |
| R052 | DeriveKey: Object Type | 1 | §6.1.14, Table 205, Object Type row (L25900) | Yes | DERIVE |
| R053 | DeriveKey: Unique Identifier | 2 | §6.1.14, Table 205, Unique Identifier row (L25919) | Yes. MAY be repeated | DERIVE |
| R054 | DeriveKey: Derivation Method | 3 | §6.1.14, Table 205, Derivation Method row (L25940) | Yes | DERIVE |
| R055 | DeriveKey: Derivation Parameters | 4 | §6.1.14, Table 205, Derivation Parameters row (L25960) | Yes | DERIVE |
| R056 | DeriveKey: Attributes | 5 | §6.1.14, Table 205, Attributes row (L25980) | Yes | ATTR, DERIVE |
| R057 | Destroy: Unique Identifier | 1 | §6.1.15, Table 208, Unique Identifier row (L26154) | No | ID, OBJECT-STATE |
| R058 | DiscoverVersions: Protocol Version | 1 | §6.1.16, Table 211, Protocol Version row (L26333) | No, MAY be Repeated | VERSIONS |
| R059 | Encrypt: Unique Identifier | 1 | §6.1.17, Table 214, Unique Identifier row (L26519) | No | ID |
| R060 | Encrypt: Cryptographic Parameters | 2 | §6.1.17, Table 214, Cryptographic Parameters row (L26537) | No | CRYPTO-CONTEXT |
| R061 | Encrypt: Data | 3 | §6.1.17, Table 214, Data row (L26558) | Yes for single-part. No for multi-part. | STREAM |
| R062 | Encrypt: IV/Counter/Nonce | 4 | §6.1.17, Table 214, IV/Counter/Nonce row (L26574) | No | CRYPTO-CONTEXT |
| R063 | Encrypt: Correlation Value | 5 | §6.1.17, Table 214, Correlation Value row (L26590) | No | STREAM |
| R064 | Encrypt: Init Indicator | 6 | §6.1.17, Table 214, Init Indicator row (L26607) | No | STREAM |
| R065 | Encrypt: Final Indicator | 7 | §6.1.17, Table 214, Final Indicator row (L26623) | No | STREAM |
| R066 | Encrypt: Authenticated Encryption Additional Data | 8 | §6.1.17, Table 214, Authenticated Encryption Additional Data row (L26639) | No | AEAD |
| R067 | Export: Unique Identifier | 1 | §6.1.18, Table 217, Unique Identifier row (L26884) | No | ID |
| R068 | Export: Key Format Type | 2 | §6.1.18, Table 217, Key Format Type row (L26902) | No | WRAP-RETURN |
| R069 | Export: Key Wrap Type | 3 | §6.1.18, Table 217, Key Wrap Type row (L26918) | No | WRAP-RETURN |
| R070 | Export: Key Compression Type | 4 | §6.1.18, Table 217, Key Compression Type row (L26934) | No | WRAP-RETURN |
| R071 | Export: Key Wrapping Specification | 5 | §6.1.18, Table 217, Key Wrapping Specification row (L26951) | No | WRAP-RETURN |
| R072 | Get: Unique Identifier | 1 | §6.1.19, Table 220, Unique Identifier row (L27191) | No | ID |
| R073 | Get: Key Format Type | 2 | §6.1.19, Table 220, Key Format Type row (L27210) | No | WRAP-RETURN |
| R074 | Get: Key Wrap Type | 3 | §6.1.19, Table 220, Key Wrap Type row (L27228) | No | WRAP-RETURN |
| R075 | Get: Key Compression Type | 4 | §6.1.19, Table 220, Key Compression Type row (L27247) | No | WRAP-RETURN |
| R076 | Get: Key Wrapping Specification | 5 | §6.1.19, Table 220, Key Wrapping Specification row (L27266) | No | WRAP-RETURN |
| R077 | GetAttributes: Unique Identifier | 1 | §6.1.20, Table 223, Unique Identifier row (L27482) | No | ID |
| R078 | GetAttributes: Attribute Reference | 2 | §6.1.20, Table 223, Attribute Reference row (L27502) | No, MAY be repeated | GET-ATTRS |
| R079 | GetAttributeList: Unique Identifier | 1 | §6.1.21, Table 226, Unique Identifier row (L27688) | No | ID |
| R080 | GetUsageAllocation: Unique Identifier | 1 | §6.1.23, Table 232, Unique Identifier row (L28060) | No | ID, USAGE |
| R081 | GetUsageAllocation: Usage Limits Count | 2 | §6.1.23, Table 232, Usage Limits Count row (L28080) | Yes | USAGE |
| R082 | Hash: Cryptographic Parameters | 1 | §6.1.24, Table 235, Cryptographic Parameters row (L28252) | Yes | Schema only |
| R083 | Hash: Data | 2 | §6.1.24, Table 235, Data row (L28270) | Yes for single-part. No for multi-part. | STREAM |
| R084 | Hash: Correlation Value | 3 | §6.1.24, Table 235, Correlation Value row (L28287) | No | STREAM |
| R085 | Hash: Init Indicator | 4 | §6.1.24, Table 235, Init Indicator row (L28305) | No | STREAM |
| R086 | Hash: Final Indicator | 5 | §6.1.24, Table 235, Final Indicator row (L28321) | No | STREAM |
| R087 | Import: Unique Identifier | 1 | §6.1.25, Table 238, Unique Identifier row (L28507) | Yes | IMPORT |
| R088 | Import: Object Type | 2 | §6.1.25, Table 238, Object Type row (L28524) | Yes | IMPORT |
| R089 | Import: Replace Existing | 3 | §6.1.25, Table 238, Replace Existing row (L28540) | No | IMPORT |
| R090 | Import: Key Wrap Type | 4 | §6.1.25, Table 238, Key Wrap Type row (L28558) | If and only if the key object is wrapped. | IMPORT |
| R091 | Import: Attributes | 5 | §6.1.25, Table 238, Attributes row (L28580) | Yes | ATTR, IMPORT |
| R092 | Import: Any Object (Section 2) | 6 | §6.1.25, Table 238, Any Object (Section 2) row (L28596) | Yes | IMPORT, KEY-OBJECT |
| R093 | Interop: Interop Function | 1 | §6.1.26, Table 241, Interop Function row (L28763) | Yes | INTEROP |
| R094 | Interop: Interop Identifier | 2 | §6.1.26, Table 241, Interop Identifier row (L28779) | Yes | INTEROP |
| R095 | JoinSplitKey: Object Type | 1 | §6.1.27, Table 244, Object Type row (L28943) | Yes | JOIN |
| R096 | JoinSplitKey: Unique Identifier | 2 | §6.1.27, Table 244, Unique Identifier row (L28959) | Yes, MAY be repeated | JOIN |
| R097 | JoinSplitKey: Secret Data Type | 3 | §6.1.27, Table 244, Secret Data Type row (L28980) | No | JOIN |
| R098 | JoinSplitKey: Attributes | 4 | §6.1.27, Table 244, Attributes row (L28996) | No | ATTR, JOIN |
| R099 | JoinSplitKey: Protection Storage Masks | 5 | §6.1.27, Table 244, Protection Storage Masks row (L29012) | No | JOIN, MASKS |
| R100 | Locate: Maximum Items | 1 | §6.1.28, Table 247, Maximum Items row (L29258) | No | LOCATE |
| R101 | Locate: Offset Items | 2 | §6.1.28, Table 247, Offset Items row (L29277) | No | LOCATE |
| R102 | Locate: Storage Status Mask | 3 | §6.1.28, Table 247, Storage Status Mask row (L29297) | No | LOCATE |
| R103 | Locate: Object Group Member | 4 | §6.1.28, Table 247, Object Group Member row (L29318) | No | LOCATE |
| R104 | Locate: Attributes | 5 | §6.1.28, Table 247, Attributes row (L29337) | Yes | ATTR, LOCATE |
| R105 | Log: Log Message | 1 | §6.1.29, Table 250, Log Message row (L29529) | Yes | Schema only |
| R106 | Login: Lease Time | 1 | §6.1.30, Table 253, Lease Time row (L29673) | No | LOGIN-RIGHTS |
| R107 | Login: Request Count | 2 | §6.1.30, Table 253, Request Count row (L29689) | No | LOGIN-RIGHTS |
| R108 | Login: Usage Limits | 3 | §6.1.30, Table 253, Usage Limits row (L29705) | No | LOGIN-RIGHTS |
| R109 | Logout: Ticket | 1 | §6.1.31, Table 256, Ticket row (L29866) | Yes | Schema only |
| R110 | Mac: Unique Identifier | 1 | §6.1.32, Table 259, Unique Identifier row (L30021) | No | ID |
| R111 | Mac: Cryptographic Parameters | 2 | §6.1.32, Table 259, Cryptographic Parameters row (L30039) | No | CRYPTO-CONTEXT |
| R112 | Mac: Data | 3 | §6.1.32, Table 259, Data row (L30060) | Yes for single-part. No for multi-part. | STREAM |
| R113 | Mac: Correlation Value | 4 | §6.1.32, Table 259, Correlation Value row (L30077) | No | STREAM |
| R114 | Mac: Init Indicator | 5 | §6.1.32, Table 259, Init Indicator row (L30095) | No | STREAM |
| R115 | Mac: Final Indicator | 6 | §6.1.32, Table 259, Final Indicator row (L30111) | No | STREAM |
| R116 | MacVerify: Unique Identifier | 1 | §6.1.33, Table 262, Unique Identifier row (L30331) | No | ID |
| R117 | MacVerify: Cryptographic Parameters | 2 | §6.1.33, Table 262, Cryptographic Parameters row (L30350) | No | CRYPTO-CONTEXT |
| R118 | MacVerify: Data | 3 | §6.1.33, Table 262, Data row (L30371) | No | STREAM |
| R119 | MacVerify: MAC Data | 4 | §6.1.33, Table 262, MAC Data row (L30387) | Yes for single-part. No for multi-part. | STREAM |
| R120 | MacVerify: Correlation Value | 5 | §6.1.33, Table 262, Correlation Value row (L30404) | No | STREAM |
| R121 | MacVerify: Init Indicator | 6 | §6.1.33, Table 262, Init Indicator row (L30422) | No | STREAM |
| R122 | MacVerify: Final Indicator | 7 | §6.1.33, Table 262, Final Indicator row (L30438) | No | STREAM |
| R123 | ModifyAttribute: Unique Identifier | 1 | §6.1.34, Table 265, Unique Identifier row (L30648) | No | ID |
| R124 | ModifyAttribute: Current Attribute | 2 | §6.1.34, Table 265, Current Attribute row (L30668) | No | ATTRIBUTE-OPS |
| R125 | ModifyAttribute: New Attribute | 3 | §6.1.34, Table 265, New Attribute row (L30687) | Yes | ATTRIBUTE-OPS |
| R126 | ObtainLease: Unique Identifier | 1 | §6.1.35, Table 268, Unique Identifier row (L30866) | No | ID, OBJECT-STATE |
| R127 | Pkcs11: PKCS#11 Interface | 1 | §6.1.37, Table 273, PKCS#11 Interface row (L31184) | No | PKCS |
| R128 | Pkcs11: PKCS#11 Function | 2 | §6.1.37, Table 273, PKCS#11 Function row (L31203) | Yes | PKCS |
| R129 | Pkcs11: Correlation Value | 3 | §6.1.37, Table 273, Correlation Value row (L31222) | No | PKCS |
| R130 | Pkcs11: PKCS#11 Input Parameters | 4 | §6.1.37, Table 273, PKCS#11 Input Parameters row (L31241) | No | PKCS |
| R131 | Poll: Asynchronous Correlation Value | 1 | §6.1.38, Table 276, Asynchronous Correlation Value row (L31495) | Yes | ASYNC |
| R132 | Process: Asynchronous Correlation Value | 1 | §6.1.39, Table 278, Asynchronous Correlation Value row (L31618) | Yes | ASYNC |
| R133 | Query: Query Function | 1 | §6.1.40, Table 282, Query Function row (L32025) | Yes, MAY be Repeated | QUERY |
| R134 | Query: Object Groups | 2 | §6.1.40, Table 282, Object Groups row (L32043) | No | QUERY |
| R135 | QueryAsyncRequests: Asynchronous Correlation Values | 1 | §6.1.41, Table 285, Asynchronous Correlation Values row (L32456) | No | ASYNC |
| R136 | QueryAsyncRequests: Operations | 2 | §6.1.41, Table 285, Operations row (L32475) | No | ASYNC |
| R137 | Recover: Unique Identifier | 1 | §6.1.42, Table 288, Unique Identifier row (L32650) | No | ID, OBJECT-STATE |
| R138 | Register: Object Type | 1 | §6.1.43, Table 291, Object Type row (L32832) | Yes | Schema only |
| R139 | Register: Attributes | 2 | §6.1.43, Table 291, Attributes row (L32850) | Yes | ATTR |
| R140 | Register: Any Object (Section 2) | 3 | §6.1.43, Table 291, Any Object (Section 2) row (L32871) | Yes | KEY-OBJECT |
| R141 | Register: Protection Storage Masks | 4 | §6.1.43, Table 291, Protection Storage Masks row (L32890) | No | MASKS |
| R142 | Revoke: Unique Identifier | 1 | §6.1.44, Table 295, Unique Identifier row (L33176) | No | ID |
| R143 | Revoke: Revocation Reason | 2 | §6.1.44, Table 295, Revocation Reason row (L33195) | Yes | REVOKE |
| R144 | Revoke: Compromise Occurrence Date | 3 | §6.1.44, Table 295, Compromise Occurrence Date row (L33213) | No | REVOKE |
| R145 | Recertify: Unique Identifier | 1 | §6.1.45, Table 300, Unique Identifier row (L33625) | No | ID, RECERT |
| R146 | Recertify: Certificate Request Unique Identifier | 2 | §6.1.45, Table 300, Certificate Request Unique Identifier row (L33645) | No | RECERT |
| R147 | Recertify: Certificate Request Type | 3 | §6.1.45, Table 300, Certificate Request Type row (L33663) | No | RECERT |
| R148 | Recertify: Certificate Request Value | 4 | §6.1.45, Table 300, Certificate Request Value row (L33682) | No | RECERT |
| R149 | Recertify: Offset | 5 | §6.1.45, Table 300, Offset row (L33700) | No | RECERT |
| R150 | Recertify: Attributes | 6 | §6.1.45, Table 300, Attributes row (L33720) | No | ATTR, RECERT |
| R151 | Recertify: Protection Storage Masks | 7 | §6.1.45, Table 300, Protection Storage Masks row (L33740) | No | RECERT, MASKS |
| R152 | Rekey: Unique Identifier | 1 | §6.1.46, Table 305, Unique Identifier row (L34222) | No | ID, REKEY |
| R153 | Rekey: Offset | 2 | §6.1.46, Table 305, Offset row (L34242) | No | REKEY |
| R154 | Rekey: Attributes | 3 | §6.1.46, Table 305, Attributes row (L34262) | No | ATTR, REKEY |
| R155 | Rekey: Protection Storage Masks | 4 | §6.1.46, Table 305, Protection Storage Masks row (L34280) | No | REKEY, MASKS |
| R156 | RekeyKeyPair: Private Key Unique Identifier | 1 | §6.1.47, Table 310, Private Key Unique Identifier row (L34757) | No | ID, REKEY |
| R157 | RekeyKeyPair: Offset | 2 | §6.1.47, Table 310, Offset row (L34776) | No | REKEY |
| R158 | RekeyKeyPair: Common Attributes | 3 | §6.1.47, Table 310, Common Attributes row (L34796) | No | ATTR, REKEY |
| R159 | RekeyKeyPair: Private Key Attributes | 4 | §6.1.47, Table 310, Private Key Attributes row (L34817) | No | ATTR, REKEY |
| R160 | RekeyKeyPair: Public Key Attributes | 5 | §6.1.47, Table 310, Public Key Attributes row (L34836) | No | ATTR, REKEY |
| R161 | RekeyKeyPair: Common Protection Storage Masks | 6 | §6.1.47, Table 310, Common Protection Storage Masks row (L34855) | No | REKEY, MASKS |
| R162 | RekeyKeyPair: Private Protection Storage Masks | 7 | §6.1.47, Table 310, Private Protection Storage Masks row (L34874) | No | REKEY, MASKS |
| R163 | RekeyKeyPair: Public Protection Storage Masks | 8 | §6.1.47, Table 310, Public Protection Storage Masks row (L34898) | No | REKEY, MASKS |
| R164 | ReProvision: Certificate Request | 1 | §6.1.48, Table 313, Certificate Request row (L35135) | No | REPROVISION |
| R165 | ReProvision: Certificate | 2 | §6.1.48, Table 313, Certificate row (L35153) | No | REPROVISION |
| R166 | RngRetrieve: Data Length | 1 | §6.1.49, Table 316, Data Length row (L35321) | Yes | RNG |
| R167 | RngSeed: Data | 1 | §6.1.50, Table 319, Data row (L35498) | Yes | RNG |
| R168 | SetAttribute: Unique Identifier | 1 | §6.1.51, Table 322, Unique Identifier row (L35667) | No | ID |
| R169 | SetAttribute: New Attribute | 2 | §6.1.51, Table 322, New Attribute row (L35685) | Yes | ATTRIBUTE-OPS |
| R170 | SetConstraints: Constraints | 1 | §6.1.52, Table 325, Constraints row (L35849) | Yes | CONSTRAINTS-DEFAULTS |
| R171 | SetDefaults: Defaults Information | 1 | §6.1.53, Table 328, Defaults Information row (L36013) | No | CONSTRAINTS-DEFAULTS |
| R172 | Sign: Unique Identifier | 1 | §6.1.55, Table 334, Unique Identifier row (L36352) | No | ID |
| R173 | Sign: Cryptographic Parameters | 2 | §6.1.55, Table 334, Cryptographic Parameters row (L36370) | No | CRYPTO-CONTEXT |
| R174 | Sign: Data | 3 | §6.1.55, Table 334, Data row (L36391) | Yes for single-part, unless Digested Data is supplied.. No for multi-part. | STREAM |
| R175 | Sign: Digested Data | 4 | §6.1.55, Table 334, Digested Data row (L36407) | No | STREAM |
| R176 | Sign: Correlation Value | 5 | §6.1.55, Table 334, Correlation Value row (L36423) | No | STREAM |
| R177 | Sign: Init Indicator | 6 | §6.1.55, Table 334, Init Indicator row (L36440) | No | STREAM |
| R178 | Sign: Final Indicator | 7 | §6.1.55, Table 334, Final Indicator row (L36456) | No | STREAM |
| R179 | SignatureVerify: Unique Identifier | 1 | §6.1.56, Table 337, Unique Identifier row (L36684) | No | ID |
| R180 | SignatureVerify: Cryptographic Parameters | 2 | §6.1.56, Table 337, Cryptographic Parameters row (L36703) | No | CRYPTO-CONTEXT |
| R181 | SignatureVerify: Data | 3 | §6.1.56, Table 337, Data row (L36726) | No | STREAM |
| R182 | SignatureVerify: Digested Data | 4 | §6.1.56, Table 337, Digested Data row (L36742) | No | STREAM |
| R183 | SignatureVerify: Signature Data | 5 | §6.1.56, Table 337, Signature Data row (L36758) | Yes for single-part. No for multi-part. | STREAM |
| R184 | SignatureVerify: Correlation Value | 6 | §6.1.56, Table 337, Correlation Value row (L36775) | No | STREAM |
| R185 | SignatureVerify: Init Indicator | 7 | §6.1.56, Table 337, Init Indicator row (L36793) | No | STREAM |
| R186 | SignatureVerify: Final Indicator | 8 | §6.1.56, Table 337, Final Indicator row (L36809) | No | STREAM |
| R187 | Validate: Certificate | 1 | §6.1.57, Table 340, Certificate row (L37044) | No, MAY be repeated | VALIDATE |
| R188 | Validate: Unique Identifier | 2 | §6.1.57, Table 340, Unique Identifier row (L37062) | No, MAY be repeated | VALIDATE |
| R189 | Validate: Validity Date | 3 | §6.1.57, Table 340, Validity Date row (L37081) | No | VALIDATE |

| R190 | Create: nested Object Attribute multiplicity | Nested in Attributes | §4, Table 46; §6.1.8, Table 186 | N/A (nested attribute cardinality) | Reject repeated instances of an attribute whose source-defined multiplicity is single when the known Object Type resolves the rule; preserve repeatable and unknown attributes. |

## Follow-up: verification Data requiredness (R118 / R181)

This note clarifies the STREAM group: R118/R181 participate in the **framing/correlation** audit, but membership in that group does not establish mandatory original Data for verification operations.

**Literal general scope versus specific sources.** The unqualified wording in §6.1 paragraph at HTML L22498–L22499 can literally encompass the Data-tagged fields R118 and R181: it says Data is required in cryptographic operations unless Init or Final is true. It does not explicitly exclude verification operations. However, §6.1.33 Table 262, Data row L30371, and §6.1.56 Table 337, Data row L36726, each explicitly mark original Data REQUIRED=No. The operation prose reinforces that distinction: §6.1.33 L30281 and §6.1.56 L36630 say the request MAY contain the original data for algorithms that need it. Therefore, it is not source-backed to state that the general sentence definitively excludes R118/R181; nor is it source-backed to silently override the explicit optional rows with a universal original-Data requirement. This is a **general/specific scope ambiguity**, retained without changing the approved validation boundary. The general sentence uses lowercase “required”; the operation-specific MAY wording is uppercase. This difference is relevant evidence, but it does not by itself constitute an official erratum or a resolved interpretation.

**Distinct fields must stay distinct.** R119 MAC Data (Table 262 L30387), R182 Digested Data (Table 337 L36742), and R183 Signature Data (Table 337 L36758) have their own identities. The general paragraph names Data, not those fields, and supplies no substitution equation or rule making them mandatory on intermediate multipart requests. Original Data R118/R181 must not be conflated with the MAC/signature being checked. Table 337 permits optional Digested Data; it does not state a Data/Digested Data XOR or universal requirement for one of them. For Sign, Table 334 supplies an explicit Data-unless-Digested-Data alternative; no identical rule may be copied into Signature Verify without its own source.

**Safe locally decidable checks for 0051.** Apply each exact payload table's type/order/singleton cardinality; apply the shared framing/correlation rules where the supplied fields make the condition decidable. For a clearly single-part MAC Verify request, require MAC Data under Table 262. For clearly single-part Signature Verify, require Signature Data under Table 337. Do not introduce a new rejection solely because optional original Data R118/R181 is absent, including an intermediate multipart request, while the general/specific scope is unresolved. Whether a particular verification algorithm needs original Data may depend on server-side key/algorithm parameters and is not a universal stateless requirement. Never turn REQUIRED=No for multi-part into a prohibition on supplying Data, MAC Data, or Signature Data. Existing approved project gates must be retained and separately identified as project behavior rather than newly claimed OASIS mandates.

**Disposition.** The table/prose-supported optional original-Data capability is the safe preservation boundary; a universal verification Data-presence rejection is **not currently justified** by unambiguous source evidence. Record this as A-VERIFY-DATA if tracking it separately. It is not a closure or expansion of KMIPKIT-DISC-045, whose approved operation set remains Encrypt, Decrypt, Hash, MAC, and Sign. No production changes or tracked-file edits are made by this note.
