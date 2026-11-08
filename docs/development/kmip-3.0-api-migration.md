# KMIP 3.0 API corrections before 1.0

KMIPKit corrects two public model names that assigned meanings absent from the
pinned KMIP 3.0 tables. Apply these changes before building against a 1.0
release.

## Object Type

`ObjectType::CREDENTIAL` and `ObjectType::Credential` previously assigned
`0x0000000D` the broad name “Credential”. KMIP 3.0 §11.35, Table 582 assigns
that value specifically to **Password Credential**. Use
`ObjectType::PASSWORD_CREDENTIAL` or `ObjectType::PasswordCredential` instead.
The other assigned credential Object Types now have named constants:
`DEVICE_CREDENTIAL`, `ONE_TIME_PASSWORD_CREDENTIAL`, and
`HASHED_PASSWORD_CREDENTIAL`.

## Object Class

The former closed enum variants `CryptographicObject`, `SystemObject`, and
`MetadataObject` did not match KMIP 3.0 §11.34, Table 581. The corrected
`ObjectClass` is an open raw-value wrapper with assigned constants `USER` and
`SYSTEM`; unknown values remain representable.

```rust
use kmipkit::protocol::{ObjectClass, ObjectType};

let object_type = ObjectType::PASSWORD_CREDENTIAL;
let object_class = ObjectClass::USER;
let future_class = ObjectClass::from_raw(0x8000_0001);
assert_eq!(future_class.to_raw(), 0x8000_0001);
```

## Link normalization

The pre-1.0 `LinkType` API previously inferred KMIP 2.1 values and KMIP 3.0
attribute tags from enum discriminants and contiguous ranges. Those numbers do
not share an ordering: KMIP 2.1 §11.28, Table 465 assigns Link Type values
`0x101`–`0x10E`, while KMIP 3.0 §§4.35.1–4.35.20 and §11.58 assign different
tags. The [KMIPKIT-0071 link mapping](../../specs/071-kmip-3-0-draft-support/spec.md)
documents the 14 exact semantic equivalents and their KMIP 3.0 tags. Use the
corrected semantic variants and explicit lookup methods. The
former `PreviousKey`, `NextKey`, `ParentObject`, `ChildObject`, and `SplitKey`
variants are removed; use `Previous`, `Next`, `Parent`, and `Child` for the
corresponding shared values. `ReplacementObject`, `ReplacedObject`,
`Pkcs12Certificate`, and `Pkcs12Password` are now represented explicitly.

`normalize_to_v2` returns an error for the six KMIP 3.0-only links
(Credential, Password, Group, Split Key Base, Joined Split Key Parts, and
Certificate Request). It also returns an error for KMIP 3.0 Reference/Name
Reference identifier forms that have no lossless KMIP 2.1 representation.
KMIP 2.1 permits Text String identifiers, but KMIP 3.0 §4.35 does not list
Text String for these link attributes, so conversion to KMIP 3.0 also rejects
that encoding. Enumeration and Integer identifiers round-trip unchanged. The
original item remains unchanged when normalization fails.

These corrected values and models do not claim complete KMIP 3.0 protocol or
profile support. See KMIPKIT-0069 through KMIPKIT-0071 for the scoped
multi-version contract.
