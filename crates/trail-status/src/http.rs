//! A minimal HTTP/1.1 client over `std::net`, for the contract checkers.
//! One request per connection (`Connection: close`), so reading to EOF is the
//! whole response.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// A response's status code and body.
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// Send one request to `addr` (e.g. `127.0.0.1:8080`).
pub fn request(
    addr: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<Response, String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| format!("connect {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw).map_err(|e| e.to_string())?;
    parse_response(&raw)
}

fn parse_response(raw: &str) -> Result<Response, String> {
    let status = raw
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("no status line in {raw:?}"))?;
    let body = raw
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    Ok(Response { status, body })
}

/// Wait until something accepts connections on `addr`, up to `timeout`.
pub fn wait_for_server(addr: &str, timeout: Duration) -> Result<(), String> {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if TcpStream::connect(addr).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(format!("nothing listening on {addr} after {timeout:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_and_body() {
        let r = parse_response("HTTP/1.1 404 Not Found\r\nContent-Length: 2\r\n\r\n{}").unwrap();
        assert_eq!(r.status, 404);
        assert_eq!(r.body, "{}");
    }

    #[test]
    fn refuses_garbage() {
        assert!(parse_response("not http").is_err());
    }
}
