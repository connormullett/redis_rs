use std::io::{self, Read};
use std::net;

use io::Write;
use net::TcpStream;

use crate::enums::RedisError;
use crate::parse::{parse_command, parse_response};
use crate::response::Response;

/// Holds connection information for the redis server
pub struct Connection {
    /// The server host
    pub host: String,
    /// The server port
    pub port: u32,
}

impl Connection {
    /// Create a new Connection
    pub fn new(host: String, port: u32) -> Connection {
        Connection { host, port }
    }

    /// Create a default connection on localhost:6379. Good for testing local connections
    pub fn default() -> Connection {
        Connection {
            host: String::from("127.0.0.1"),
            port: 6379,
        }
    }

    /// Send a raw request string to the redis server
    pub fn send_raw_request(&self, command: String) -> Result<Response, RedisError> {
        let request = parse_command(command);
        let response = self.write(request)?;
        let response: Response = parse_response(response)?;

        Ok(response)
    }

    /// Append `value` to the value related to `key`. Returns number of bytes read as integer
    pub fn append(&self, key: &str, value: &str) -> Result<Response, RedisError> {
        let request = format!("append {} '{}'", key, value);
        let response = self.send_raw_request(request)?;
        Ok(response)
    }

    /// Send a get request to fetch a specified `key`. Returns the value as a Response
    pub fn get(&self, key: &str) -> Result<Response, RedisError> {
        let request = format!("get {}", &key);
        let response = self.send_raw_request(request)?;
        Ok(response)
    }

    /// Send an echo request. This is great for health checking the server
    /// Returns the string sent as a simple string
    pub fn echo(&self, string: &str) -> Result<Response, RedisError> {
        let request = format!("echo {}", &string);
        let response = self.send_raw_request(request)?;
        Ok(response)
    }

    /// Send a set request to create a new `key` with value `value`.
    /// Returns OK as a SimpleString on success
    pub fn set(&self, key: &str, value: &str) -> Result<Response, RedisError> {
        let request = format!(
            "*3\r\n$3\r\nset\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
            key.chars().count(),
            key,
            value.chars().count(),
            value
        );

        let response_data = self.write(request)?;
        let response = parse_response(response_data)?;

        Ok(response)
    }

    /// Ping the server. The response data should be PONG
    /// Returns "PONG" as a SimpleString on success
    pub fn ping(&self) -> Result<Response, RedisError> {
        let request = String::from("PING");
        let response = self.send_raw_request(request)?;
        Ok(response)
    }

    /// Delete keys from the server
    /// Returns the number of keys removed
    pub fn delete(&self, keys: Vec<&str>) -> Result<Response, RedisError> {
        let mut request = String::from("del ");

        for key in keys {
            request.push_str(&format!("{} ", key));
        }

        let response = self.send_raw_request(request)?;

        Ok(response)
    }

    #[doc(hidden)]
    fn write(&self, request: String) -> Result<String, RedisError> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = match TcpStream::connect(addr) {
            Ok(s) => s,
            Err(_) => {
                return Err(RedisError::SocketConnectionError(
                    "Can not connect to socket".to_string(),
                ));
            }
        };

        let _ = match stream.write(request.as_bytes()) {
            Ok(value) => value,
            Err(_) => {
                return Err(RedisError::SocketConnectionError(
                    "Error writing to stream".to_string(),
                ));
            }
        };

        let mut response = String::new();

        // response is guaranteed to be less than 512 bytes
        let mut buffer: [u8; 512] = [0; 512];
        let _ = match stream.read(&mut buffer) {
            Ok(value) => value,
            Err(_) => {
                return Err(RedisError::SocketConnectionError(
                    "Error reading from stream".to_string(),
                ))
            }
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
    use crate::response::Response;

    #[test]
    fn test_connection_new() {
        let host = String::from("127.0.0.1");
        let port = 6379;
        let c = connection::Connection::new(host.clone(), port);

        assert_eq!(c.host, host);
        assert_eq!(c.port, port);
    }

    #[test]
    fn test_append() {
        let client = connection::Connection::new(String::from("127.0.0.1"), 6379);
        let response = client.append("BAZ", "FOO").unwrap();

        assert_eq!(response, Response::Integer(9));
    }

    #[test]
    fn test_send_get() {
        let client = connection::Connection::new(String::from("127.0.0.1"), 6379);

        let response = client.get("FOO").unwrap();

        if let Response::SimpleString(value) = &response {
            println!("value {}", value);
        }

        assert_eq!(response, Response::BulkString(String::from("BAR")));
    }

    #[test]
    fn test_ping() {
        let client = connection::Connection::new("127.0.0.1".to_string(), 6379);
        let response = client.ping().unwrap();
        assert_eq!(response, Response::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_send_set() {
        let client = connection::Connection::new("127.0.0.1".to_string(), 6379);

        let response = client.set("BAZ", "QUUX").unwrap();

        assert_eq!(response, Response::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_delete() {
        let client = connection::Connection::new("127.0.0.1".to_string(), 6379);
        let key = vec!["val"];
        let value = "value";
        let _ = client.set(key[0], value);
        let response = client.delete(key).unwrap();

        assert_eq!(response, Response::Integer(1));
    }

    #[test]
    fn test_multi_delete() {
        let client = connection::Connection::new("127.0.0.1".to_string(), 6379);
        let _ = client.set("bar1", "bar");
        let _ = client.set("bar2", "bar");

        let keys = vec!["bar1", "bar2"];

        let response = client.delete(keys).unwrap();

        assert_eq!(response, Response::Integer(2));
    }

    #[test]
    fn test_connection_send() {
        let host = "127.0.0.1";
        let port = 6379;
        let c = connection::Connection::new(host.to_string(), port);
        let command = String::from("PING");

        let response = c.send_raw_request(command).unwrap();

        assert_eq!(response, Response::SimpleString(String::from("PONG")));
    }

    #[test]

    fn test_parse_send_quoted_set() {
        let connection = connection::Connection::new("127.0.0.1".to_string(), 6379);
        let response = connection.set("myvalue", "a custom value").unwrap();

        assert_eq!(response, Response::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_connection_write() {
        let connection = connection::Connection::new("127.0.0.1".to_string(), 6379);
        let command = "PING\r\n";

        let response = connection.write(command.to_string());
        assert!(response.is_ok());
    }

    #[test]
    fn test_connection_test_multi_word_requests() {
        let connection = connection::Connection::new("127.0.0.1".to_string(), 6379);

        let set_request = String::from("SET FOO BAR");
        let get_request = String::from("GET FOO");

        let set_response = connection.send_raw_request(set_request);
        assert!(set_response.is_ok());

        let get_response = connection.send_raw_request(get_request).unwrap();
        assert_eq!(get_response, Response::BulkString(String::from("BAR")));
    }
}
