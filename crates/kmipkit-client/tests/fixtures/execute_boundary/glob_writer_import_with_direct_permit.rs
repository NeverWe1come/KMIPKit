use crate::private_wire_writer::*;

impl Client {
    fn execute(&mut self, request: TypedRequest) {
        let permit = OperationEncodingPermit { _private: () };
        encode_for_execute(request, permit);
    }
}
