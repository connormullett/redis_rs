use std::{error::Error, fmt};

#[derive(Debug)]
/// An error type that describes a client error
pub enum RedisError {
    ParseError,
    SocketConnectionError,
}

#[doc(hidden)]
impl Error for RedisError {}

#[doc(hidden)]
impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured")
    }
}
