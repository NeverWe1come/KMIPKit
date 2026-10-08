use kmipkit_client::ClientBatchResponse;

fn caller_cannot_read_the_raw_response(response: ClientBatchResponse) {
    let _ = response.as_bytes();
}

fn main() {}
