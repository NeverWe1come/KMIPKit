pub enum RequestInput {
    Typed(kmipkit_ttlv::Item),
    Raw(&'static [u8]),
}
