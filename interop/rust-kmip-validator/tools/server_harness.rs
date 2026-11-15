//! Keep the pinned test-server's current-thread runtime polling while it serves.
#![forbid(unsafe_code)]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let (keepalive, shutdown) = tokio::sync::oneshot::channel();
    let result = kmipkit_test_server::config::run_cli_with_shutdown(&args, Some(shutdown)).await;
    drop(keepalive);
    result
}
