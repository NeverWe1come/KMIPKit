fn record_correlation(value: &[u8]) {
    tracing::info!(asynchronous_correlation_value = ?value);
}
