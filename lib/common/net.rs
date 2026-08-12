// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Networking primitives for My Language
//!
//! Provides TCP, UDP sockets and HTTP client functionality.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket, ToSocketAddrs};
use std::time::Duration;

/// TCP client connection
pub struct TcpClient {
    stream: TcpStream,
}

impl TcpClient {
    /// Connect to a TCP server
    pub fn connect(addr: &str) -> Result<Self, String> {
        TcpStream::connect(addr)
            .map(|stream| TcpClient { stream })
            .map_err(|e| format!("Failed to connect: {}", e))
    }

    /// Set read timeout in milliseconds
    pub fn set_read_timeout(&mut self, ms: u64) -> Result<(), String> {
        self.stream
            .set_read_timeout(Some(Duration::from_millis(ms)))
            .map_err(|e| format!("Failed to set timeout: {}", e))
    }

    /// Set write timeout in milliseconds
    pub fn set_write_timeout(&mut self, ms: u64) -> Result<(), String> {
        self.stream
            .set_write_timeout(Some(Duration::from_millis(ms)))
            .map_err(|e| format!("Failed to set timeout: {}", e))
    }

    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<usize, String> {
        self.stream
            .write(data)
            .map_err(|e| format!("Failed to send: {}", e))
    }

    /// Send a string
    pub fn send_str(&mut self, data: &str) -> Result<usize, String> {
        self.send(data.as_bytes())
    }

    /// Receive data into buffer, returns bytes read
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, String> {
        self.stream
            .read(buffer)
            .map_err(|e| format!("Failed to receive: {}", e))
    }

    /// Receive all data until EOF
    pub fn recv_all(&mut self) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();
        self.stream
            .read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to receive: {}", e))?;
        Ok(buffer)
    }

    /// Receive a string (assumes UTF-8)
    pub fn recv_string(&mut self) -> Result<String, String> {
        let bytes = self.recv_all()?;
        String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {}", e))
    }

    /// Close the connection
    pub fn close(self) {
        // Stream closes automatically on drop
    }
}

/// TCP server listener
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    /// Bind to an address
    pub fn bind(addr: &str) -> Result<Self, String> {
        TcpListener::bind(addr)
            .map(|listener| TcpServer { listener })
            .map_err(|e| format!("Failed to bind: {}", e))
    }

    /// Accept a connection (blocking)
    pub fn accept(&self) -> Result<TcpClient, String> {
        self.listener
            .accept()
            .map(|(stream, _)| TcpClient { stream })
            .map_err(|e| format!("Failed to accept: {}", e))
    }

    /// Get the local address
    pub fn local_addr(&self) -> Result<String, String> {
        self.listener
            .local_addr()
            .map(|addr| addr.to_string())
            .map_err(|e| format!("Failed to get address: {}", e))
    }
}

/// UDP socket
pub struct Udp {
    socket: UdpSocket,
}

impl Udp {
    /// Bind to an address
    pub fn bind(addr: &str) -> Result<Self, String> {
        UdpSocket::bind(addr)
            .map(|socket| Udp { socket })
            .map_err(|e| format!("Failed to bind: {}", e))
    }

    /// Send data to an address
    pub fn send_to(&self, data: &[u8], addr: &str) -> Result<usize, String> {
        self.socket
            .send_to(data, addr)
            .map_err(|e| format!("Failed to send: {}", e))
    }

    /// Send a string to an address
    pub fn send_str_to(&self, data: &str, addr: &str) -> Result<usize, String> {
        self.send_to(data.as_bytes(), addr)
    }

    /// Receive data (blocking)
    pub fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, String), String> {
        self.socket
            .recv_from(buffer)
            .map(|(size, addr)| (size, addr.to_string()))
            .map_err(|e| format!("Failed to receive: {}", e))
    }

    /// Set read timeout in milliseconds
    pub fn set_read_timeout(&self, ms: u64) -> Result<(), String> {
        self.socket
            .set_read_timeout(Some(Duration::from_millis(ms)))
            .map_err(|e| format!("Failed to set timeout: {}", e))
    }

    /// Get the local address
    pub fn local_addr(&self) -> Result<String, String> {
        self.socket
            .local_addr()
            .map(|addr| addr.to_string())
            .map_err(|e| format!("Failed to get address: {}", e))
    }
}

/// HTTP client for making requests
pub struct HttpClient {
    timeout_ms: u64,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new() -> Self {
        HttpClient {
            timeout_ms: 30000, // 30 second default
        }
    }

    /// Set request timeout in milliseconds
    pub fn set_timeout(&mut self, ms: u64) {
        self.timeout_ms = ms;
    }

    /// Make a GET request
    pub fn get(&self, url: &str) -> Result<HttpResponse, String> {
        self.request("GET", url, None, &[])
    }

    /// Make a POST request with body
    pub fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse, String> {
        self.request("POST", url, Some(("application/octet-stream", body.len())), body)
    }

    /// Make a POST request with JSON body
    pub fn post_json(&self, url: &str, json: &str) -> Result<HttpResponse, String> {
        let body = json.as_bytes();
        self.request("POST", url, Some(("application/json", body.len())), body)
    }

    /// Make a generic HTTP request
    fn request(
        &self,
        method: &str,
        url: &str,
        content_type: Option<(&str, usize)>,
        body: &[u8],
    ) -> Result<HttpResponse, String> {
        // Parse URL
        let url = url.strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .ok_or_else(|| "URL must start with http:// or https://".to_string())?;

        let (host, path) = url.split_once('/')
            .map(|(h, p)| (h, format!("/{}", p)))
            .unwrap_or((url, "/".to_string()));

        // Connect
        let addr = if host.contains(':') {
            host.to_string()
        } else {
            format!("{}:80", host)
        };

        let mut client = TcpClient::connect(&addr)?;
        client.set_read_timeout(self.timeout_ms)?;
        client.set_write_timeout(self.timeout_ms)?;

        // Build request
        let mut request = format!("{} {} HTTP/1.1\r\nHost: {}\r\n", method, path, host);

        if let Some((content_type, length)) = content_type {
            request.push_str(&format!("Content-Type: {}\r\n", content_type));
            request.push_str(&format!("Content-Length: {}\r\n", length));
        }

        request.push_str("Connection: close\r\n\r\n");

        // Send request
        client.send_str(&request)?;

        if !body.is_empty() {
            client.send(body)?;
        }

        // Receive response
        let response_str = client.recv_string()?;
        HttpResponse::parse(&response_str)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP response
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpResponse {
    /// Parse an HTTP response
    fn parse(response: &str) -> Result<Self, String> {
        let mut lines = response.lines();

        // Parse status line
        let status_line = lines.next().ok_or("Empty response")?;
        let parts: Vec<&str> = status_line.splitn(3, ' ').collect();

        if parts.len() < 2 {
            return Err("Invalid status line".to_string());
        }

        let status = parts[1].parse::<u16>()
            .map_err(|_| "Invalid status code".to_string())?;
        let status_text = parts.get(2).unwrap_or(&"").to_string();

        // Parse headers
        let mut headers = Vec::new();
        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                headers.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        // Rest is body
        let body = lines.collect::<Vec<&str>>().join("\n");

        Ok(HttpResponse {
            status,
            status_text,
            headers,
            body,
        })
    }

    /// Check if response is successful (2xx status)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Get a header value by name (case-insensitive)
    pub fn header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_response_parse() {
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello";
        let parsed = HttpResponse::parse(response).unwrap();
        assert_eq!(parsed.status, 200);
        assert_eq!(parsed.body, "Hello");
        assert!(parsed.is_success());
    }
}
