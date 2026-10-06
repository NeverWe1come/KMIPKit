fn hidden_submit(request: TypedRequest) {
    let permit = OperationEncodingPermit::mint();
    private_wire_writer::encode(request, permit);
}
