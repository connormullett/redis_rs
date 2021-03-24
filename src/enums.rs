use std::{error, fmt};

pub type RedisResult<T> = Result<T, RedisError>;

pub type RedisError = Box<dyn error::Error>;

#[derive(Debug, Clone)]
pub struct RedisParseError {
    pub contents: String,
}

impl fmt::Display for RedisParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured while parsing")
    }
}

impl error::Error for RedisParseError {}
