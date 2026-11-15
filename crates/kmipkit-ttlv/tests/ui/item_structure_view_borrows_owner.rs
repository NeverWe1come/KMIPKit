use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value};

fn nested_view_cannot_outlive_its_item() -> StructureView<'static> {
    let tag = RawTag::new(0x0042_0173)
        .expect("the test tag fits the KMIP width")
        .try_checked()
        .expect("the test tag is assigned in KMIP 2.1");
    let item = Item::new(tag, Value::structure(Structure::new()))
        .expect("the checked tag and Structure form an item");
    item.as_structure()
        .expect("the item contains a Structure")
}

fn main() {
    let _view = nested_view_cannot_outlive_its_item();
}
