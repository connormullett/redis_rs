use std::io::{self, Read};
use std::net;

use io::Write;
use net::TcpStream;

use crate::enums::{RedisError, Response};
use crate::parse::{parse_command, parse_response};

pub struct Connection<'a> {
    host: &'a str,
    port: u32,
}

#[allow(dead_code)]
impl<'a> Connection<'a> {
    pub fn new(host: &'a str, port: u32) -> Connection {
        Connection { host, port }
    }

    pub fn send(&self, command: &str) -> Result<Response, RedisError> {
        let request = parse_command(command)?;
        let response = self.write(request)?;
        let response = parse_response(&response)?;

        Ok(response)
    }

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
    fn test_connection_test_multi_word_requests() {
        let connection = connection::Connection::new("127.0.0.1", 6379);

        let set_request = "SET\r\nFOO\r\nBAR";
        let get_request = "GET\r\nFOO";

        let set_response = connection.send(set_request);
        assert!(set_response.is_ok());

        let get_response = connection.send(get_request).unwrap();
        assert_eq!(get_response.data, "BAR");
    }
}
