use crate::enums::{Commands, RedisError};

#[allow(dead_code)]
pub fn parse_command(command: &str) -> Result<String, RedisError> {
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

#[allow(dead_code)]
pub fn parse_response(response: &str) -> Result<String, RedisError> {
    Ok(response.to_string())
}

#[cfg(test)]
mod test {
    use crate::parse;
    #[test]
    fn test_parse_command() {
        let command = String::from("GET FOO");

        let parsed_command = parse::parse_command(&command).unwrap();

        assert_eq!("*2\r\n$3\r\nGET\r\n$3\r\nFOO\r\n", parsed_command);
    }
}
