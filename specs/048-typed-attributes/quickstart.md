# KMIPKIT-0048 Quickstart Scenarios

The exact public names are exercised by the Rust consumer tests. Intended use:

```rust
let name = Attribute::<typed::Name>::new(name_structure)?;
let mut attributes = AttributeSet::new();
attributes.try_push_typed(name)?;

for value in attributes.typed::<typed::Name>() {
    value.with_value(|structure| {
        // Navigate the borrowed StructureView without changing its child order.
        let _children = structure.children();
    })?;
}

let comment = Attribute::<typed::Comment>::new("primary signing key".to_owned())?;
let new_attribute = NewAttribute::from_typed(comment);
```

The method names above are fixed by
[`contracts/typed-attribute-api.md`](contracts/typed-attribute-api.md) and will
be compiled by the consumer test. Tests must cover one scalar, one repeated
attribute, one Structure-valued attribute, an open Enumeration, invalid
tag/type pairs, and the generic fallback path. They must never assert that a
remote server accepts an attribute.
