use std::io::{self, Read};
use std::net;

use io::Write;
use net::TcpStream;

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
            Err(_) => return Err(RedisError::SocketConnectionError),
        };

        let _ = match stream.write(request.as_bytes()) {
            Ok(value) => value,
            Err(_) => return Err(RedisError::SocketConnectionError),
        };

        let mut response = String::new();
        let _ = match stream.read_to_string(&mut response) {
            Ok(value) => value,
            Err(_) => return Err(RedisError::SocketConnectionError),
        };

        Ok(response)
    }
}

#[derive(Debug)]
pub enum RedisError {
    ParseError,
    InvalidCommandError,
    SocketConnectionError,
}

custom_derive! {
    #[allow(non_camel_case_types)]
    #[derive(EnumFromStr)]
    pub enum Commands {
        get,
        set,
        echo
    }
}

#[allow(dead_code)]
fn parse_command(command: &str) -> Result<String, RedisError> {
    let mut output = String::new();
    let tokens: Vec<&str> = command.split(' ').collect();

    if tokens.is_empty() {
        return Err(RedisError::ParseError);
    }

    let command = tokens[0];

    if command.to_lowercase().parse::<Commands>().is_err() {
        return Err(RedisError::InvalidCommandError);
    }

    let token_count = tokens.len();
    output.push_str(&format!("*{}\r\n", token_count));

    for token in tokens {
        let length = token.len();
        output.push_str(&format!("${}\r\n{}\r\n", length, token));
    }

    Ok(output)
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
}
