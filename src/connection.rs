use std::io::{self, BufRead, BufReader};
use std::net;

use io::Write;
use net::TcpStream;

use crate::enums::RedisError;
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

    pub fn send(&self, command: &str) -> Result<String, RedisError> {
        let request = parse_command(command)?;
        let response = self.write(request)?;
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

        let mut reader = BufReader::new(stream);

        let mut response = String::new();
        let _ = match reader.read_line(&mut response) {
            Ok(value) => value,
            Err(e) => {
                println!("ERROR :: {}", e);
                return Err(RedisError::SocketConnectionError);
            }
        };

        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use crate::connection;
    #[test]
    fn test_parse_command() {
        let command = String::from("GET FOO");

        let parsed_command = connection::parse_command(&command).unwrap();

        assert_eq!("*2\r\n$3\r\nGET\r\n$3\r\nFOO\r\n", parsed_command);
    }

    #[test]
    fn test_write() {
        let connection = connection::Connection::new("127.0.0.1", 6379);
        let command = "PING";

        let response = connection.send(command);
        assert!(response.is_ok())
    }
}
