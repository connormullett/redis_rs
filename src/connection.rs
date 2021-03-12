use std::io::{self, Read};

use io::Write;

use crate::enums::RedisError;
use crate::parse::{parse_command, parse_response};
use crate::response::Response;

/// Holds connection information for the redis server
pub struct Connection<T> {
    /// The server host
    pub host: String,
    /// The server port
    pub port: u32,
    /// A stream like object that implements Read + Write.
    /// This must be a Tcp connection or Unix like socket
    pub stream: T,
}

impl<T> Connection<T>
where
    T: Read + Write,
{
    /// Create a new Connection
    pub fn new(host: String, port: u32, stream: T) -> Connection<T> {
        Connection { host, port, stream }
    }

    /// Send a raw request string to the redis server
    pub fn send_raw_request(&mut self, command: &str) -> Result<Response, RedisError> {
        let request = parse_command(command);
        let _ = self.write(&request)?;
        let response = self.read()?;
        let response: Response = parse_response(&response)?;

        Ok(response)
    }

    /// Append `value` to the value related to `key`. Returns number of bytes read as integer
    pub fn append(&mut self, key: &str, value: &str) -> Result<Response, RedisError> {
        let request = format!("append {} '{}'", key, value);
        let command = parse_command(&request);
        let _ = self.write(&command)?;
        let response = self.read()?;
        let parsed_response = parse_response(&response)?;
        Ok(parsed_response)
    }

    /// Send a get request to fetch a specified `key`. Returns the value as a Response
    pub fn get(&mut self, key: &str) -> Result<Response, RedisError> {
        let request = format!("get {}", &key);
        let response = self.send_raw_request(&request)?;
        Ok(response)
    }

    /// Send an echo request. This is great for health checking the server
    /// Returns the string sent as a simple string
    pub fn echo(&mut self, string: &str) -> Result<Response, RedisError> {
        let request = format!("echo {}", &string);
        let response = self.send_raw_request(&request)?;
        Ok(response)
    }

    /// Send a set request to create a new `key` with value `value`.
    /// Returns OK as a SimpleString on success
    pub fn set(&mut self, key: &str, value: &str) -> Result<Response, RedisError> {
        let request = format!(
            "*3\r\n$3\r\nset\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
            key.chars().count(),
            key,
            value.chars().count(),
            value
        );

        let _ = self.write(&request)?;
        let response = self.read()?;
        let response = parse_response(&response)?;
        Ok(response)
    }

    /// Ping the server. The response data should be PONG
    /// Returns "PONG" as a SimpleString on success
    pub fn ping(&mut self) -> Result<Response, RedisError> {
        let request = "PING";
        let response = self.send_raw_request(request)?;
        Ok(response)
    }

    /// Delete keys from the server
    /// Returns the number of keys removed
    pub fn delete(&mut self, keys: Vec<&str>) -> Result<Response, RedisError> {
        let mut request = String::from("del ");

        for key in keys {
            request.push_str(&format!("{} ", key));
        }

        let response = self.send_raw_request(&request)?;

        Ok(response)
    }

    pub fn copy(&mut self, source: &str, destination: &str) -> Result<Response, RedisError> {
        let request = format!("copy {} {}", source, destination);

        let response = self.send_raw_request(&request)?;

        Ok(response)
    }

    fn write(&mut self, request: &str) -> Result<(), RedisError> {
        let _ = match self.stream.write(request.as_bytes()) {
            Ok(value) => value,
            Err(e) => {
                return Err(RedisError::SocketConnectionError(e.to_string()));
            }
        };

        Ok(())
    }

    fn read(&mut self) -> Result<String, RedisError> {
        let mut response = String::new();

        let mut buffer: [u8; 512] = [0; 512];
        let _ = match self.stream.read(&mut buffer) {
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
    use std::net::TcpStream;

    use crate::response::Response;
    use crate::{connection, enums::RedisError};

    use super::Connection;

    fn create_connection(addr: String) -> Result<TcpStream, RedisError> {
        match TcpStream::connect(addr) {
            Ok(s) => Ok(s),
            Err(_) => {
                return Err(RedisError::SocketConnectionError(
                    "Can not connect to server".to_string(),
                ));
            }
        }
    }

    #[test]
    fn test_passing_tcp_stream_to_connection() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut client = Connection::new("127.0.0.1".to_string(), 6379, stream);

        let response = client.set("new", "bar").unwrap();

        assert_eq!(response, Response::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_connection_new() {
        let host = String::from("127.0.0.1");
        let port = 6379;
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let c = connection::Connection::new(host.clone(), port, stream);

        assert_eq!(c.host, host);
        assert_eq!(c.port, port);
    }

    #[test]
    fn test_append() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();

        let mut client = connection::Connection::new(String::from("127.0.0.1"), 6379, stream);
        let _ = client.set("append_value", "value");
        let response = client.append("append_value", "foo").unwrap();

        assert_eq!(response, Response::Integer(10));
    }

    #[test]
    fn test_send_get() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();

        let mut client = connection::Connection::new(String::from("127.0.0.1"), 6379, stream);

        let response = client.get("FOO").unwrap();

        if let Response::SimpleString(value) = &response {
            println!("value {}", value);
        }

        assert_eq!(response, Response::BulkString(String::from("BAR")));
    }

    #[test]
    fn test_ping() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut client = connection::Connection::new("127.0.0.1".to_string(), 6379, stream);
        let response = client.ping().unwrap();
        assert_eq!(response, Response::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_send_set() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut client = connection::Connection::new("127.0.0.1".to_string(), 6379, stream);
        let response = client.set("BAZ", "QUUX").unwrap();

        assert_eq!(response, Response::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_delete() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut client = connection::Connection::new("127.0.0.1".to_string(), 6379, stream);
        let key = vec!["val"];
        let value = "value";
        let _ = client.set(key[0], value);
        let response = client.delete(key).unwrap();

        assert_eq!(response, Response::Integer(1));
    }

    #[test]
    fn test_multi_delete() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut client = connection::Connection::new("127.0.0.1".to_string(), 6379, stream);
        let _ = client.set("bar1", "bar");
        let _ = client.set("bar2", "bar");

        let keys = vec!["bar1", "bar2"];

        let response = client.delete(keys).unwrap();

        assert_eq!(response, Response::Integer(2));
    }

    #[test]
    fn test_copy() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut client = connection::Connection::new("127.0.0.1".to_string(), 6379, stream);

        let _ = client.set("bar1", "bar");
        let _ = client.delete(vec!["new_bar"]);

        let response = client.copy("bar1", "new_bar").unwrap();

        let response_value = match response {
            Response::Integer(x) => x,
            _ => panic!(),
        };

        assert!(response_value <= 1);
    }

    #[test]
    fn test_connection_send() {
        let host = "127.0.0.1";
        let port = 6379;

        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut c = connection::Connection::new(host.to_string(), port, stream);

        let response = c.send_raw_request("PING").unwrap();

        assert_eq!(response, Response::SimpleString(String::from("PONG")));
    }

    #[test]

    fn test_parse_send_quoted_set() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut connection = connection::Connection::new("127.0.0.1".to_string(), 6379, stream);
        let response = connection.set("myvalue", "a custom value").unwrap();

        assert_eq!(response, Response::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_connection_write() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut connection = connection::Connection::new("127.0.0.1".to_string(), 6379, stream);
        let command = "PING\r\n";

        let response = connection.write(command);
        assert!(response.is_ok());
    }

    #[test]
    fn test_connection_test_multi_word_requests() {
        let stream = create_connection("127.0.0.1:6379".to_string()).unwrap();
        let mut connection = connection::Connection::new("127.0.0.1".to_string(), 6379, stream);

        let set_request = String::from("SET FOO BAR");
        let get_request = String::from("GET FOO");

        let set_response = connection.send_raw_request(&set_request);
        assert!(set_response.is_ok());

        let get_response = connection.send_raw_request(&get_request).unwrap();
        assert_eq!(get_response, Response::BulkString(String::from("BAR")));
    }
}
