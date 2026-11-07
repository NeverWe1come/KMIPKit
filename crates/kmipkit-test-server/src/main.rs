//! Standalone KMIP 1.0-2.1 test server binary.
#![forbid(unsafe_code)]

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = kmipkit_test_server::config::run_cli_with_shutdown(&args, None).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
