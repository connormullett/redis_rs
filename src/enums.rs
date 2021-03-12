use std::fmt;

#[derive(Debug)]
/// An error type that describes a client error
pub enum RedisError {
    /// Error variant used when a request/response fails to parse
    ParseError(String),
    /// Error variant used when an error occurs when reading/writing from/to the Connection stream
    SocketConnectionError(String),
}

#[doc(hidden)]
impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured")
    }
}
