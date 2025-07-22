use std::pin::Pin;

use simvar::switchy::unsync::io::AsyncReadExt;

use crate::Error;

/// Reads an HTTP response from a stream until the complete response is received.
///
/// # Errors
///
/// Returns an error if reading from the stream fails or if the response is malformed.
pub async fn read_http_response(
    response: &mut String,
    mut stream: Pin<Box<impl AsyncReadExt>>,
) -> Result<Option<String>, Error> {
    let mut buf = [0_u8; 4096];

    Ok(loop {
        let count = match stream.read(&mut buf).await {
            Ok(count) => count,
            Err(e) => {
                log::error!("read_http_response: failed to read from stream: {e:?}");
                break None;
            }
        };
        if count == 0 {
            log::debug!("read_http_response: received empty response");
            break None;
        }
        log::trace!("read count={count}");
        let value = String::from_utf8_lossy(&buf[..count]).to_string();
        response.push_str(&value);

        // Look for end of HTTP response (double CRLF)
        if response.contains("\r\n\r\n") {
            break Some(response.clone());
        }
    })
}

/// Parses an HTTP response string and extracts the status code and body.
///
/// # Errors
///
/// Returns an error if the response format is invalid, the status line is malformed,
/// or the status code cannot be parsed as a valid u16.
pub fn parse_http_response(response: &str) -> Result<(u16, String), Error> {
    let lines: Vec<&str> = response.split("\r\n").collect();

    if lines.is_empty() {
        return Err(Error::IO(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Empty HTTP response",
        )));
    }

    // Parse status line (e.g., "HTTP/1.1 200 OK")
    let status_line = lines[0];
    let parts: Vec<&str> = status_line.split_whitespace().collect();

    if parts.len() < 2 {
        return Err(Error::IO(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid HTTP status line",
        )));
    }

    let status_code: u16 = parts[1].parse().map_err(|_| {
        Error::IO(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid HTTP status code",
        ))
    })?;

    // Find the body (after the empty line)
    let mut body = String::new();
    let mut in_body = false;

    for line in lines {
        if in_body {
            if !body.is_empty() {
                body.push_str("\r\n");
            }
            body.push_str(line);
        } else if line.is_empty() {
            in_body = true;
        }
    }

    Ok((status_code, body))
}
