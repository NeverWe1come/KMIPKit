# Data Model: KMIP 2.1 Query and Ping

## QueryFunction

An open KMIP Query Function enumeration value. Named values are Query Operations (1), Query Objects (2), Query Server Information (3), Query Application Namespaces (4), Query Extension List (5), Query Extension Map (6), Query Attestation Types (7), Query RNGs (8), Query Validations (9), Query Profiles (10), Query Capabilities (11), Query Client Registration Methods (12), Query Defaults Information (13), and Query Storage Protection Masks (14), from §11.44, Table 476. Valid extension/future values remain representable. Reserved values follow the shared KMIP enumeration policy.

## QueryRequest

The Query request contains one or more ordered Query Function values and optional Object Groups, following §6.1.40, Table 282. Object Groups is the structure in §7.23, Table 375, and contains zero or more repeated `Object Group` members. Each member is an `Object Group` attribute with a Text String value (§4.35, Tables 99–100). The model preserves member order and values; it does not deduplicate repeated functions or attributes, insert defaults, or invent object groups. The §11.33 Object Group Member enumeration is a separate Locate request option and does not type this structure member.

## QueryResponse

A successful Query response is represented as either an empty operation payload or a structured response. Section 6.1.40 says the payload is empty when no values are available, while Table 283 marks Protection Storage Masks as required and allows its list to be empty. This source conflict is tracked as open `KMIPKIT-DISC-045`; the decoder accepts both forms and makes no server-conformance determination. For the structured form, each member's required/optional and repeatable status follows Table 283.

| Member | Wire shape represented by the typed model |
| --- | --- |
| Operation | Repeated open Operation values |
| Object Type | Repeated open Object Type values |
| Vendor Identification | Optional text value |
| Server Information | Optional structure/item |
| Application Namespace | Optional repeated text values |
| Extension Information | Optional repeated structures/items |
| Attestation Type | Optional repeated open values |
| RNG Parameters | Optional repeated structures/items |
| Profile Information | Optional repeated structures/items |
| Validation Information | Optional repeated structures/items |
| Capability Information | Optional repeated structures/items |
| Client Registration Method | Optional repeated open values |
| Defaults Information | Optional structure/item |
| Protection Storage Masks | Required in a structured response; the server may return an empty list |

Absent optional fields remain absent; received repetitions and order remain observable. Unknown, vendor, and future Items are retained. Values report the server's response and do not imply local policy or authorization.

## Ping

Ping request and response operation payloads are empty under §6.1.36, Tables 271–272. The operation result is carried by the common KMIP batch/result model.

## Shared outcome and ownership

- Query and Ping outcomes are correlated by the existing batch model.
- Success/failure, Result Reason/Message, and delivery evidence use the common contracts. The catalog records no asynchronous response contract for Ping or Query.
- One explicit call performs no more than one exchange; the caller controls every later request.
- Decoding observes configured TTLV size, depth, and element limits before allocation.
- Formatting and errors never expose raw request or response bodies.