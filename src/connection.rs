use std::io::{self, Read};
use std::net;

use io::Write;
use net::TcpStream;

use crate::enums::{RedisError, Response};
use crate::parse::{parse_command, parse_response};

/// Holds connection information for the redis server
pub struct Connection<'a> {
    host: &'a str,
    port: u32,
}

#[allow(dead_code)]
impl<'a> Connection<'a> {
    /// Create a new Connection
    pub fn new(host: &'a str, port: u32) -> Connection {
        Connection { host, port }
    }

    /// Send a raw request string to the redis server
    pub fn send(&self, command: &str) -> Result<Response, RedisError> {
        let request = parse_command(command)?;
        let response = self.write(request)?;
        let response = parse_response(&response)?;

        Ok(response)
    }

    /// Send a get request to fetch a specified `key`
    pub fn send_get(&self, key: &str) -> Result<Response, RedisError> {
        let request = format!("get {}", &key);
        let response = self.send(&request)?;
        Ok(response)
    }

    /// Send an echo request. This is great for health checking the server
    pub fn echo(&self, string: &str) -> Result<Response, RedisError> {
        let request = format!("echo {}", &string);
        let response = self.send(&request)?;
        Ok(response)
    }

    /// Send a set request to create a new `key` with value `value`
    pub fn send_set(&self, key: &str, value: &str) -> Result<Response, RedisError> {
        let request = format!("set {} {}", &key, &value);
        let response = self.send(&request)?;
        Ok(response)
    }

    #[doc(hidden)]
    fn write(&self, request: String) -> Result<String, RedisError> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = match TcpStream::connect(addr) {
            Ok(s) => s,
            Err(_) => {
                return Err(RedisError::SocketConnectionError);
            }
        };

        let _ = match stream.write(request.as_bytes()) {
            Ok(value) => value,
            Err(_) => {
                return Err(RedisError::SocketConnectionError);
            }
        };

        let mut response = String::new();

        // response is guaranteed to be less than 512 bytes
        let mut buffer: [u8; 512] = [0; 512];
        let _ = match stream.read(&mut buffer) {
            Ok(value) => value,
            Err(_) => return Err(RedisError::SocketConnectionError),
        };

        for char in buffer.iter() {
            response.push(*char as char);
        }

        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use crate::connection;
    use crate::enums::ResponseType;

    #[test]
    fn test_connection_new() {
        let host = "127.0.0.1";
        let port = 6379;
        let c = connection::Connection::new(host, port);

        assert_eq!(c.host, host);
        assert_eq!(c.port, port);
    }

    #[test]
    fn test_connection_send() {
        let host = "127.0.0.1";
        let port = 6379;
        let c = connection::Connection::new(host, port);
        let command = "PING";

        let response = c.send(command).unwrap();

        assert_eq!(response.data, "PONG");
    }

    #[test]
    fn test_connection_write() {
        let connection = connection::Connection::new("127.0.0.1", 6379);
        let command = "PING\r\n";

        let response = connection.write(command.to_string());
        assert!(response.is_ok());
    }

    #[test]
    fn test_connection_error_response_should_match_expected() {
        let connection = connection::Connection::new("127.0.0.1", 6379);
        let command = "list FOO";

        let response = connection.send(command).unwrap();

        assert_eq!(response.response_type, ResponseType::Error);
    }

    #[test]
    fn test_connection_test_multi_word_requests() {
        let connection = connection::Connection::new("127.0.0.1", 6379);

        let set_request = "SET FOO BAR";
        let get_request = "GET FOO";

        let set_response = connection.send(set_request);
        assert!(set_response.is_ok());

        let get_response = connection.send(get_request).unwrap();
        assert_eq!(get_response.data, "BAR");
    }
}
