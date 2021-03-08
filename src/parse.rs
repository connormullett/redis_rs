use crate::enums::RedisError;
pub use crate::response::{
    Response,
    Response::{Array, BulkString, Error, Integer, SimpleString},
};

#[doc(hidden)]
pub fn parse_command(command: String) -> String {
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
pub fn parse_response(response: String) -> Result<Response, RedisError> {
    let mut bytes = response.bytes();

    let first_byte = match bytes.next() {
        Some(value) => value,
        None => return Err(RedisError::ParseError),
    };

    let response = match first_byte as char {
        '+' => parse_simple_string(&mut bytes),
        '-' => parse_error(&mut bytes),
        ':' => parse_integer(&mut bytes),
        '$' => parse_bulk_string(&mut bytes),
        '*' => parse_array(&mut bytes),
        _ => Ok(Error(format!(
            "unexpected byte {}, in response",
            first_byte
        ))),
    };

    Ok(response?)
}

fn parse_error(bytes: &mut std::str::Bytes) -> Result<Response, RedisError> {
    let mut error_string = String::new();

    while let Some(c) = bytes.next() {
        let c = c as char;
        if let '\r' = c {
            break;
        }

        error_string.push(c);
    }

    Ok(Error(error_string))
}

fn parse_integer(bytes: &mut std::str::Bytes) -> Result<Response, RedisError> {
    let mut integer_value = String::new();

    while let Some(c) = bytes.next() {
        let c = c as char;

        if let '\r' = c {
            break;
        }

        integer_value.push(c);
    }

    let parsed_integer: i32 = match integer_value.parse() {
        Ok(value) => value,
        Err(_) => return Err(RedisError::ParseError),
    };

    Ok(Integer(parsed_integer))
}

fn parse_bulk_string(bytes: &std::str::Bytes) -> Result<Response, RedisError> {
    todo!()
}

fn parse_array(bytes: &std::str::Bytes) -> Result<Response, RedisError> {
    todo!()
}

fn parse_simple_string(bytes: &std::str::Bytes) -> Result<Response, RedisError> {
    todo!()
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

        let parsed_command = parse::parse_command(command);

        assert_eq!("*2\r\n$3\r\nGET\r\n$3\r\nFOO\r\n", parsed_command);
    }

    #[test]
    fn test_parse_quotes_handled_properly() {
        let client = Connection::new("127.0.0.1", 6379);

        let raw_request = String::from("set myvalue 'a custom value'");
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
        let data = String::from("+OK\r\n");

        let response = parse_response(data).unwrap();

        assert_eq!(response, Response::SimpleString(String::from("OK")));
    }

    #[test]
    fn test_error_response() {
        let data = String::from("-ERROR\r\n");

        let response = parse_response(data).unwrap();
        assert_eq!(response, Response::Error(String::from("ERROR")));
    }
}
