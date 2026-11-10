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

These corrected values and models do not claim complete KMIP 3.0 protocol or
profile support. See KMIPKIT-0069 through KMIPKIT-0071 for the scoped
multi-version contract.
