use std::{error, fmt};

/// Alias for `Result<T, RedisError>`.
/// This is the main return type for most
/// methods within the client.
pub type RedisResult<T> = Result<T, RedisError>;

/// A dynamic error type aliased to RedisError
pub type RedisError = Box<dyn error::Error>;

#[derive(Debug, Clone)]
/// An error that occurs while parsing requests/responses.
pub struct RedisParseError {
    pub contents: String,
}

impl fmt::Display for RedisParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured while parsing")
    }
}

impl error::Error for RedisParseError {}
