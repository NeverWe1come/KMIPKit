#[cfg(any(target_os = "windows", target_os = "linux"))]
fn conditional_submit(request: TypedRequest) {
    let permit = OperationEncodingPermit::mint();
    private_wire_writer::encode(request, permit);
}
