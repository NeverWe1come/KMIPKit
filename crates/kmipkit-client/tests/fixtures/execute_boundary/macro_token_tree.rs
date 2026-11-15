macro_rules! hidden_writer_call {
    () => {{
        let permit = OperationEncodingPermit::mint();
        private_wire_writer::encode(request, permit);
    }};
}
