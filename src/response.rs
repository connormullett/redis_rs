#[derive(Debug, PartialEq)]
/// The type the response data will be according to RESP specification
pub enum Response<'a> {
    /// Equivalent to String
    SimpleString(String),
    /// The server replied with an error
    Error(String),
    /// A numeric string that can be parsed
    Integer(i32),
    /// A string with known size
    BulkString(&'a str),
    /// An array
    Array(String),
    #[doc(hidden)]
    Base,
}
