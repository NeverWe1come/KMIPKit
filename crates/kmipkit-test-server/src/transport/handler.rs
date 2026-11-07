//! Unified connection handler for Raw TLS TTLV and HTTPS (TTLV, JSON, XML).

use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::codec::Format;
use crate::ops::Dispatcher;

/// Handles an established TLS stream. Supports both Raw TTLV frames and HTTP/1.1 POST /kmip requests.
pub async fn handle_connection<S>(mut stream: S, dispatcher: Arc<Dispatcher>) -> Result<(), String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        // Read the first byte to detect protocol without consuming excessive bytes
        let mut first_byte = [0u8; 1];
        match stream.read_exact(&mut first_byte).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Connection closed cleanly
                return Ok(());
            }
            Err(e) => return Err(format!("socket read error: {e}")),
        }

        if first_byte[0] == 0x42 {
            // Raw TTLV message (starts with tag 0x0042_xxxx)
            handle_raw_ttlv(&mut stream, first_byte[0], &dispatcher).await?;
        } else if first_byte[0] == b'P' || first_byte[0] == b'p' {
            // HTTP POST request
            let keep_alive = handle_http(&mut stream, first_byte[0], &dispatcher).await?;
            if !keep_alive {
                break;
            }
        } else {
            return Err(format!(
                "unrecognized protocol starting with byte: 0x{:02x}",
                first_byte[0]
            ));
        }
    }

    Ok(())
}

async fn handle_raw_ttlv<S>(
    stream: &mut S,
    first_byte: u8,
    dispatcher: &Dispatcher,
) -> Result<(), String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // Read the remaining 7 bytes of the 8-byte TTLV header
    let mut header_rest = [0u8; 7];
    stream
        .read_exact(&mut header_rest)
        .await
        .map_err(|e| format!("failed reading TTLV header: {e}"))?;

    let mut full_header = [0u8; 8];
    full_header[0] = first_byte;
    full_header[1..8].copy_from_slice(&header_rest);

    let payload_len = u32::from_be_bytes([
        full_header[4],
        full_header[5],
        full_header[6],
        full_header[7],
    ]) as usize;
    let padding = (8 - payload_len % 8) % 8;
    let total_to_read = payload_len + padding;

    let mut payload = vec![0u8; total_to_read];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|e| format!("failed reading TTLV payload: {e}"))?;

    let mut full_request = Vec::with_capacity(8 + total_to_read);
    full_request.extend_from_slice(&full_header);
    full_request.extend_from_slice(&payload);

    let request_item = Format::Ttlv
        .decode(&full_request)
        .map_err(|e| format!("TTLV decode error: {e}"))?;

    let response_item = dispatcher.dispatch(&request_item)?;
    let response_bytes = Format::Ttlv
        .encode(&response_item)
        .map_err(|e| format!("TTLV encode error: {e}"))?;

    stream
        .write_all(&response_bytes)
        .await
        .map_err(|e| format!("socket write error: {e}"))?;
    stream
        .flush()
        .await
        .map_err(|e| format!("socket flush error: {e}"))?;

    Ok(())
}

async fn handle_http<S>(
    stream: &mut S,
    first_byte: u8,
    dispatcher: &Dispatcher,
) -> Result<bool, String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // Read headers until \r\n\r\n
    let mut header_buf = Vec::with_capacity(1024);
    header_buf.push(first_byte);

    let mut matched_end = false;
    let mut byte = [0u8; 1];
    while !matched_end && header_buf.len() < 16 * 1024 {
        stream
            .read_exact(&mut byte)
            .await
            .map_err(|e| format!("error reading HTTP header: {e}"))?;
        header_buf.push(byte[0]);
        if header_buf.ends_with(b"\r\n\r\n") {
            matched_end = true;
        }
    }

    if !matched_end {
        return Err("HTTP header too large or truncated".to_string());
    }

    let header_str = String::from_utf8_lossy(&header_buf);
    let mut content_length = 0;
    let mut content_type = None;
    let mut connection_close = false;

    for line in header_str.lines() {
        let line_lower = line.to_ascii_lowercase();
        if let Some(val) = line_lower.strip_prefix("content-length:") {
            content_length = val.trim().parse::<usize>().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("Content-Type:") {
            content_type = Some(val.trim().to_string());
        } else if let Some(val) = line_lower.strip_prefix("content-type:") {
            content_type = Some(val.trim().to_string());
        } else if let Some(val) = line_lower.strip_prefix("connection:") {
            if val.trim() == "close" {
                connection_close = true;
            }
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        stream
            .read_exact(&mut body)
            .await
            .map_err(|e| format!("failed reading HTTP body: {e}"))?;
    }

    let format = Format::detect(content_type.as_deref(), &body);
    let request_item = format.decode(&body)?;
    let response_item = dispatcher.dispatch(&request_item)?;
    let response_bytes = format.encode(&response_item)?;

    let status_line = "HTTP/1.1 200 OK\r\n";
    let conn_header = if connection_close {
        "Connection: close\r\n"
    } else {
        "Connection: keep-alive\r\n"
    };
    let content_type_header = format!("Content-Type: {}\r\n", format.content_type());
    let content_length_header = format!("Content-Length: {}\r\n\r\n", response_bytes.len());

    let mut http_response = Vec::new();
    http_response.extend_from_slice(status_line.as_bytes());
    http_response.extend_from_slice(conn_header.as_bytes());
    http_response.extend_from_slice(content_type_header.as_bytes());
    http_response.extend_from_slice(content_length_header.as_bytes());
    http_response.extend_from_slice(&response_bytes);

    stream
        .write_all(&http_response)
        .await
        .map_err(|e| format!("HTTP response write error: {e}"))?;
    stream
        .flush()
        .await
        .map_err(|e| format!("HTTP response flush error: {e}"))?;

    Ok(!connection_close)
}
