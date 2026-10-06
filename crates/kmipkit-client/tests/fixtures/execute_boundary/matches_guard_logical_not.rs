pub fn check(value: Option<u8>) -> bool {
    matches!(value, Some(item) if !(item == 0))
}
