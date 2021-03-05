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
            Err(e) => {
                println!("ERROR :: {}", e);
                return Err(RedisError::SocketConnectionError);
            }
        };

        let _ = match stream.write(request.as_bytes()) {
            Ok(value) => value,
            Err(e) => {
                println!("ERROR :: {}", e);
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
    fn test_write() {
        let connection = connection::Connection::new("127.0.0.1", 6379);
        let command = "PING";

        let response = connection.send(command);
        println!("response :: {:?}", response);
        assert!(response.is_ok())
    }
}
