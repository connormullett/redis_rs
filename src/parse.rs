use crate::enums::{RedisError, Response, ResponseType};

pub fn parse_command(command: &str) -> Result<String, RedisError> {
    let mut output = String::new();
    let tokens: Vec<&str> = command.split(' ').collect();

    if tokens.is_empty() {
        return Err(RedisError::ParseError);
    }

    let token_count = tokens.len();
    output.push_str(&format!("*{}\r\n", token_count));

    for token in tokens {
        let length = token.len();
        output.push_str(&format!("${}\r\n{}\r\n", length, token));
    }

    Ok(output)
}

pub fn parse_response(response: &str) -> Result<Response, RedisError> {
    let mut data = String::new();

    let first_byte = match response.bytes().next() {
        Some(value) => value,
        None => return Err(RedisError::ParseError),
    };

    let response_type = match first_byte as char {
        '*' => ResponseType::Array,
        '+' => ResponseType::SimpleString,
        '-' => ResponseType::Error,
        ':' => ResponseType::Integer,
        '$' => ResponseType::BulkString,
        _ => ResponseType::Base,
    };

    if let ResponseType::Base = response_type {
        return Err(RedisError::ParseError);
    }

    let mut cur_token = String::new();
    for byte in response.bytes().skip(1) {
        if let '\r' = byte as char {
            data.push_str(&cur_token);
            cur_token.clear();
            continue;
        }

        if byte.is_ascii_alphabetic() || byte == 0x20 {
            cur_token.push(byte as char);
        }
    }

    Ok(Response::new(response_type, data))
}

#[cfg(test)]
mod test {
    use crate::{connection::Connection, enums::ResponseType};
    use parse::parse_response;

    use crate::parse;
    #[test]
    fn test_parse_command() {
        let command = String::from("GET FOO");

        let parsed_command = parse::parse_command(&command).unwrap();

        assert_eq!("*2\r\n$3\r\nGET\r\n$3\r\nFOO\r\n", parsed_command);
    }

    #[test]
    fn test_parse_quotes_handled_properly() {
        let client = Connection::new("127.0.0.1", 6379);

        let raw_request = "set myvalue 'a custom value'";
        let _ = client.send(raw_request);

        let key = "myvalue";
        let response = client.send_get(key).unwrap();

        assert_eq!(response.data, "a custom value");
    }

    #[test]
    fn test_parse_response() {
        let data = "+OK\r\n";

        let response = parse_response(data).unwrap();

        assert_eq!(response.data, "OK");
        assert_eq!(response.response_type, ResponseType::SimpleString)
    }

    #[test]
    fn test_error_response() {
        let data = "-ERROR\r\n";

        let response = parse_response(data).unwrap();
        assert_eq!(response.data, "ERROR");
        assert_eq!(response.response_type, ResponseType::Error)
    }
}
