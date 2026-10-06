pub fn encode(request: TypedRequest, permit: OperationEncodingPermit) -> Vec<u8> {
    serialize(request, permit)
}
