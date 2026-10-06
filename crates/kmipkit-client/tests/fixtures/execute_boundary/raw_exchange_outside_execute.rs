fn direct_low_level_exchange(transport: &Transport, caller_owned_request: &[u8]) {
    transport.exchange(caller_owned_request);
}
