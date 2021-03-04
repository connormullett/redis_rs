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
