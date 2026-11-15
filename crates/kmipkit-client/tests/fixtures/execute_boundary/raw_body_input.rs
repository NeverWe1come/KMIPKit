fn execute_raw(body: &[u8]) {
    private_wire_writer::encode_raw(body);
}
