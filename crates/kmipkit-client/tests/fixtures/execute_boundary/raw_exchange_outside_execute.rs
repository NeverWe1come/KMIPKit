fn direct_low_level_exchange<T: Transport>(
    transport: &mut T,
    request: &[u8],
    max_response_bytes: usize,
) {
    let _ = transport.exchange(request, max_response_bytes);
}
