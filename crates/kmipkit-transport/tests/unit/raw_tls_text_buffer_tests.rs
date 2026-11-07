use super::{ResponseBuffer, ResponseBufferDropObserver};

#[test]
fn text_response_buffer_zeroizes_partial_bytes_and_replaced_allocations() {
    const RESPONSE: [u8; 40] = [0xA5; 40];
    let observer = ResponseBufferDropObserver::new(RESPONSE.len());

    {
        let mut response = ResponseBuffer::with_empty_and_observer_for_test(observer.clone());
        for byte in RESPONSE {
            response
                .append_text_byte(byte, RESPONSE.len())
                .expect("the bounded text response grows to the exact fixture length");
        }
    }

    assert_eq!(observer.initialized_len(), RESPONSE.len());
    assert!(observer.contained_nonzero_bytes());
    assert!(observer.initialized_range_was_zero());
    assert!(observer.replaced_allocations_were_zero());
}
