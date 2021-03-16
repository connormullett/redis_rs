use crate::enums::RedisError;
pub use crate::response::{
    Response,
    Response::{Array, BulkString, Error, Integer, SimpleString},
};
use std::str::Bytes;

pub fn parse_command(command: &str) -> Result<String, RedisError> {
    let mut tokens = Vec::<String>::new();
    let mut cur_token = String::new();
    let mut quoted = false;

    for item in command.as_bytes() {
        let item = *item as char;
        match item {
            ' ' if !quoted => {
                tokens.push(cur_token.clone());
                cur_token.clear()
            }
            '\'' if !quoted => {
                quoted = true;
            }
            '\'' if quoted => {
                quoted = false;
                tokens.push(cur_token.clone());
                cur_token.clear();
            }
            _ => cur_token.push(item),
        }
    }

    if !cur_token.is_empty() {
        tokens.push(cur_token);
    }

    if quoted && command.contains('\'') {
        return Err(RedisError::ParseError(String::from(
            "Quoted string is not closed (mismatch quotes)",
        )));
    }

    let mut output = String::new();

    output.push_str(&format!("*{}\r\n", tokens.len()));

    for token in tokens {
        let length = token.len();
        output.push_str(&format!("${}\r\n{}\r\n", length, token));
    }

    Ok(output)
}

#[doc(hidden)]
pub fn parse_response(response: &str) -> Result<Response, RedisError> {
    let mut bytes = response.bytes();

    let first_byte = match bytes.next() {
        Some(value) => value,
        None => {
            return Err(RedisError::ParseError(
                "Error reading response data".to_string(),
            ))
        }
    };

    let response = match first_byte as char {
        '+' => parse_simple_string(&mut bytes),
        '-' => parse_error(&mut bytes),
        ':' => parse_integer(&mut bytes),
        '$' => parse_bulk_string(&mut bytes),
        '*' => parse_array(&mut bytes),
        _ => Err(RedisError::ParseError(format!(
            "unexpected byte {}, in response",
            first_byte
        ))),
    }?;

    Ok(response)
}

#[doc(hidden)]
fn parse_error(bytes: &mut std::str::Bytes) -> Result<Response, RedisError> {
    let error_string = read_to_carriage_return(bytes);

    Ok(Error(error_string))
}

#[doc(hidden)]
fn parse_integer(bytes: &mut Bytes) -> Result<Response, RedisError> {
    let integer_value = read_to_carriage_return(bytes);

    let parsed_integer: i32 = match integer_value.parse() {
        Ok(value) => value,
        Err(_) => {
            return Err(RedisError::ParseError(format!(
                "Error parsing {} as integer",
                integer_value
            )))
        }
    };

    Ok(Integer(parsed_integer))
}

#[doc(hidden)]
fn parse_bulk_string(bytes: &mut Bytes) -> Result<Response, RedisError> {
    let integer_value = read_to_carriage_return(bytes);
    let mut string = String::new();

    let mut num_bytes: i32 = match integer_value.parse() {
        Ok(value) => value,
        Err(_) => {
            return Err(RedisError::ParseError(format!(
                "Error parsing {} to integer",
                integer_value.clone()
            )))
        }
    };

    for byte in bytes.skip(1) {
        if num_bytes == 0 {
            break;
        }

        string.push(byte as char);

        num_bytes -= 1;
    }

    Ok(BulkString(string))
}

#[doc(hidden)]
fn parse_array(_bytes: &mut Bytes) -> Result<Response, RedisError> {
    todo!()
}

#[doc(hidden)]
fn parse_simple_string(bytes: &mut Bytes) -> Result<Response, RedisError> {
    let string = read_to_carriage_return(bytes);
    Ok(SimpleString(string))
}

#[doc(hidden)]
fn read_to_carriage_return(bytes: &mut Bytes) -> String {
    let mut string = String::new();

    for c in bytes {
        let c = c as char;

        if let '\r' = c {
            break;
        }

        string.push(c);
    }

    string
}

#[cfg(test)]
mod test {
    use std::net::TcpStream;

    use crate::response::Response;
    use crate::{connection::Connection, enums::RedisError};
    use parse::parse_response;

    fn create_connection(addr: &str) -> Result<TcpStream, RedisError> {
        match TcpStream::connect(addr) {
            Ok(s) => Ok(s),
            Err(_) => {
                return Err(RedisError::SocketConnectionError(
                    "Can not connect to server".to_string(),
                ));
            }
        }
    }

    const HOST: &'static str = "127.0.0.1";
    const PORT: u16 = 6379;
    const ADDR: &'static str = "127.0.0.1:6379";

    use crate::parse;

    #[test]
    fn test_parse_command() {
        let parsed_command = parse::parse_command("GET FOO").unwrap();

        assert_eq!("*2\r\n$3\r\nGET\r\n$3\r\nFOO\r\n", parsed_command);
    }

    #[test]
    fn test_parse_quotes_handled_properly() {
        let stream = create_connection(ADDR).unwrap();
        let mut client = Connection::new(HOST, PORT, stream);

        let _ = client.send_raw_request("set myvalue 'a custom value'");

        let key = "myvalue";
        let response = client.get(key).unwrap();

        assert_eq!(
            response,
            Response::BulkString(String::from("a custom value"))
        );
    }

    #[test]
    fn test_parse_response() {
        let data = String::from("+OK\r\n");

        let response = parse_response(&data).unwrap();

        assert_eq!(response, Response::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_error_response() {
        let data = String::from("-ERROR\r\n");

        let response = parse_response(&data).unwrap();
        assert_eq!(response, Response::Error(String::from("ERROR")));
    }
}
