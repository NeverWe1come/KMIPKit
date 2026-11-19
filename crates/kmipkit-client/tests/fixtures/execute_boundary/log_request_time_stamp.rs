fn record_request_time_stamp(request_time_stamp: DateTime) {
    tracing::debug!(request_time_stamp = ?request_time_stamp);
}
