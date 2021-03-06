use std::{
    io::{self, Read},
    net::TcpStream,
};

use io::Write;

use crate::parse::{parse_command, parse_response};
use crate::{enums::RedisResult, response::RedisResponse};

/// Holds connection information for the redis server
pub struct Connection<T> {
    /// The server host
    pub host: String,
    /// The server port
    pub port: u16,
    /// A stream like object that implements Read + Write.
    /// This must be a Tcp connection or Unix like socket
    pub stream: T,
}

impl<T> Connection<T>
where
    T: Read + Write,
{
    /// Create a new Connection
    pub fn new(host: &str, port: u16, stream: T) -> Connection<T> {
        Connection {
            host: host.to_string(),
            port,
            stream,
        }
    }

    pub fn new_tcp(host: &str, port: u16) -> RedisResult<Connection<TcpStream>> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(addr)?;
        Ok(Connection {
            host: host.to_string(),
            port,
            stream,
        })
    }

    /// Send a raw request string to the redis server
    pub fn send_raw_request(&mut self, command: &str) -> RedisResult<RedisResponse> {
        let request = parse_command(command)?;
        let _ = self.write(&request)?;
        let response = self.read()?;
        let response = parse_response(&response)?;

        Ok(response)
    }

    /// Append `value` to the value related to `key`. Returns number of bytes read as integer
    pub fn append(&mut self, key: &str, value: &str) -> RedisResult<RedisResponse> {
        let request = format!("append {} '{}'", key, value);
        let command = parse_command(&request)?;
        let _ = self.write(&command)?;
        let response = self.read()?;
        let parsed_response = parse_response(&response)?;
        Ok(parsed_response)
    }

    /// Send a get request to fetch a specified `key`. Returns the value as a Response
    pub fn get(&mut self, key: &str) -> RedisResult<RedisResponse> {
        let request = format!("get {}", &key);
        let response = self.send_raw_request(&request)?;
        Ok(response)
    }

    /// Send an echo request. This is great for health checking the server
    /// Returns the string sent as a simple string
    pub fn echo(&mut self, string: &str) -> RedisResult<RedisResponse> {
        let request = format!("echo {}", &string);
        let response = self.send_raw_request(&request)?;
        Ok(response)
    }

    /// Send a set request to create a new `key` with value `value`.
    /// Returns OK as a SimpleString on success
    pub fn set(&mut self, key: &str, value: &str) -> RedisResult<RedisResponse> {
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
    pub fn ping(&mut self) -> RedisResult<RedisResponse> {
        let request = "PING";
        let response = self.send_raw_request(request)?;
        Ok(response)
    }

    /// Delete keys from the server
    /// Returns the number of keys removed
    pub fn delete(&mut self, keys: Vec<&str>) -> RedisResult<RedisResponse> {
        let mut request = String::from("del ");

        for key in keys {
            request.push_str(&format!("{} ", key));
        }

        let response = self.send_raw_request(&request)?;

        Ok(response)
    }

    pub fn copy(&mut self, source: &str, destination: &str) -> RedisResult<RedisResponse> {
        let request = format!("copy {} {}", source, destination);

        let response = self.send_raw_request(&request)?;

        Ok(response)
    }

    fn write(&mut self, request: &str) -> RedisResult<()> {
        let _ = self.stream.write(request.as_bytes())?;
        Ok(())
    }

    fn read(&mut self) -> RedisResult<String> {
        let mut response = String::new();

        let mut buffer: [u8; 512] = [0; 512];
        let _ = self.stream.read(&mut buffer)?;

        for char in buffer.iter() {
            response.push(*char as char);
        }

        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use std::net::TcpStream;

    use crate::connection;
    use crate::{enums::RedisResult, response::RedisResponse};

    use super::Connection;

    fn create_connection(addr: &str) -> RedisResult<TcpStream> {
        Ok(TcpStream::connect(addr)?)
    }

    const HOST: &'static str = "127.0.0.1";
    const PORT: u16 = 6379;
    const ADDR: &'static str = "127.0.0.1:6379";

    #[test]
    fn test_passing_tcp_stream_to_connection() {
        let stream = create_connection(ADDR).unwrap();
        let mut client = Connection::new(HOST, PORT, stream);

        let response = client.set("new", "bar").unwrap();

        assert_eq!(response, RedisResponse::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_reuse_connection() {
        let stream = create_connection(ADDR).unwrap();
        let mut client = Connection::new(HOST, PORT, stream);

        let _ = client.get("FOO").unwrap();
        let _ = client.get("FOO").unwrap();
        let _ = client.get("FOO").unwrap();
    }

    #[test]
    fn test_connection_new() {
        let host = "127.0.0.1";
        let port = 6379;
        let stream = create_connection(ADDR).unwrap();
        let c = connection::Connection::new(host, port, stream);

        assert_eq!(c.host, host);
        assert_eq!(c.port, port);
    }

    #[test]
    fn test_append() {
        let stream = create_connection(ADDR).unwrap();

        let mut client = connection::Connection::new(HOST, PORT, stream);
        let _ = client.set("append_value", "value");
        let response = client.append("append_value", "foo").unwrap();

        assert_eq!(response, RedisResponse::Integer(8));
    }

    #[test]
    fn test_send_get() {
        let stream = create_connection(ADDR).unwrap();

        let mut client = connection::Connection::new(HOST, PORT, stream);

        let response = client.get("FOO").unwrap();

        assert_eq!(response, RedisResponse::BulkString(String::from("BAR")));
    }

    #[test]
    fn test_ping() {
        let stream = create_connection(ADDR).unwrap();
        let mut client = connection::Connection::new(HOST, PORT, stream);
        let response = client.ping().unwrap();
        assert_eq!(response, RedisResponse::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_send_set() {
        let stream = create_connection(ADDR).unwrap();
        let mut client = connection::Connection::new(HOST, PORT, stream);
        let response = client.set("BAZ", "QUUX").unwrap();

        assert_eq!(response, RedisResponse::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_delete() {
        let stream = create_connection(ADDR).unwrap();
        let mut client = connection::Connection::new(HOST, PORT, stream);
        let key = vec!["val"];
        let value = "value";
        let _ = client.set(key[0], value);
        let response = client.delete(key).unwrap();

        assert_eq!(response, RedisResponse::Integer(1));
    }

    #[test]
    fn test_multi_delete() {
        let stream = create_connection(ADDR).unwrap();
        let mut client = connection::Connection::new(HOST, PORT, stream);
        let _ = client.set("bar1", "bar");
        let _ = client.set("bar2", "bar");

        let keys = vec!["bar1", "bar2"];

        let response = client.delete(keys).unwrap();

        assert_eq!(response, RedisResponse::Integer(2));
    }

    #[test]
    fn test_copy() {
        let stream = create_connection(ADDR).unwrap();
        let mut client = connection::Connection::new(HOST, PORT, stream);

        let _ = client.set("bar1", "bar");
        let _ = client.delete(vec!["new_bar"]);

        let response = client.copy("bar1", "new_bar").unwrap();

        let response_value = match response {
            RedisResponse::Integer(x) => x,
            _ => panic!(),
        };

        assert!(response_value <= 1);
    }

    #[test]
    fn test_send_raw_request() {
        let stream = create_connection(ADDR).unwrap();
        let mut c = connection::Connection::new(HOST, PORT, stream);

        let response = c.send_raw_request("PING").unwrap();

        assert_eq!(response, RedisResponse::SimpleString(String::from("PONG")));
        assert_eq!(
            c.send_raw_request("set mynewvalue 'a great new value'")
                .unwrap(),
            RedisResponse::SimpleString(String::from("OK"))
        );
    }

    #[test]

    fn test_parse_send_quoted_set() {
        let stream = create_connection(ADDR).unwrap();
        let mut connection = connection::Connection::new(HOST, PORT, stream);
        let response = connection.set("myvalue", "a custom value").unwrap();

        assert_eq!(response, RedisResponse::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_connection_write() {
        let stream = create_connection(ADDR).unwrap();
        let mut connection = connection::Connection::new(HOST, PORT, stream);
        let command = "PING\r\n";

        let response = connection.write(command);
        assert!(response.is_ok());
    }

    #[test]
    fn test_connection_test_multi_word_requests() {
        let stream = create_connection(ADDR).unwrap();
        let mut connection = connection::Connection::new(HOST, PORT, stream);

        let set_request = String::from("SET FOO BAR");
        let get_request = String::from("GET FOO");

        let set_response = connection.send_raw_request(&set_request);
        assert!(set_response.is_ok());

        let get_response = connection.send_raw_request(&get_request).unwrap();
        assert_eq!(get_response, RedisResponse::BulkString(String::from("BAR")));
    }
}
