use crate::private_wire_writer::encode as renamed_encode;

fn execute(request: TypedRequest, permit: OperationEncodingPermit) {
    renamed_encode(request, permit);
}
