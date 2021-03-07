use crate::enums::RedisError;
use crate::response::Response;

#[doc(hidden)]
pub fn parse_command(command: &str) -> String {
    let mut output = String::new();
    let tokens: Vec<&str> = command.split(' ').collect();

    output.push_str(&format!("*{}\r\n", tokens.len()));

    for token in tokens {
        let length = token.len();
        output.push_str(&format!("${}\r\n{}\r\n", length, token));
    }

    output
}

#[doc(hidden)]
pub fn parse_response(response: &str) -> Result<Response, RedisError> {
    let mut data = String::new();

    let mut bytes = response.bytes();

    let _first_byte = match bytes.next() {
        Some(value) => value,
        None => return Err(RedisError::ParseError),
    };

    let mut cur_token = String::new();

    for byte in bytes {
        if let '\r' = byte as char {
            data.push_str(&cur_token);
            cur_token.clear();
            continue;
        }

        if byte.is_ascii_alphabetic() || byte == 0x20 {
            cur_token.push(byte as char);
        }
    }

    Ok(Response::Base)
}

#[cfg(test)]
mod test {
    use crate::connection::Connection;
    use crate::response::Response;
    use parse::parse_response;

    use crate::parse;
    #[test]
    fn test_parse_command() {
        let command = String::from("GET FOO");

        let parsed_command = parse::parse_command(&command);

        assert_eq!("*2\r\n$3\r\nGET\r\n$3\r\nFOO\r\n", parsed_command);
    }

    #[test]
    fn test_parse_quotes_handled_properly() {
        let client = Connection::new("127.0.0.1", 6379);

        let raw_request = "set myvalue 'a custom value'";
        let _ = client.send_raw_request(raw_request);

        let key = "myvalue";
        let response = client.get(key).unwrap();

        assert_eq!(
            response,
            Response::SimpleString(String::from("a custom value"))
        );
    }

    #[test]
    fn test_parse_response() {
        let data = "+OK\r\n";

        let response = parse_response(data).unwrap();

        assert_eq!(response, Response::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_error_response() {
        let data = "-ERROR\r\n";

        let response = parse_response(data).unwrap();
        assert_eq!(response, Response::Error(String::from("ERROR")));
    }
}
