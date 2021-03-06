use std::{error::Error, fmt};

#[derive(Debug)]
pub enum RedisError {
    ParseError,
    SocketConnectionError,
}

impl Error for RedisError {}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured")
    }
}

#[derive(Debug, PartialEq)]
pub enum ResponseType {
    SimpleString,
    Error,
    Integer,
    BulkString,
    Array,
    Base,
}
