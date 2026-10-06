fn submit(request: TypedRequest, permit: OperationEncodingPermit) {
    private_wire_writer::encode(request, permit);
}
