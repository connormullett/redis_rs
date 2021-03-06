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
/// The type the response data will be according to RESP specification
pub enum ResponseType {
    /// Equivalent to String
    SimpleString,
    /// The server replied with an error
    Error,
    /// A numeric string that can be parsed
    Integer,
    /// A string with known size
    BulkString,
    /// An array
    Array,
    #[doc(hidden)]
    Base,
}
