use std::{error::Error, fmt};

custom_derive! {
    #[allow(non_camel_case_types)]
    #[derive(EnumFromStr)]
    pub enum Commands {
        get,
        set,
        echo,
        ping
    }
}

#[derive(Debug)]
pub enum RedisError {
    ParseError,
    InvalidCommandError,
    SocketConnectionError,
}

impl Error for RedisError {}

impl fmt::Display for RedisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured")
    }
}

#[derive(PartialEq)]
pub enum ResponseType {
    SimpleString,
    Error,
    Integer,
    BulkString,
    Array,
    Base,
}

pub struct Response {
    pub response_type: ResponseType,
    pub data: String,
}

impl Response {
    pub fn new(response_type: ResponseType, data: String) -> Response {
        Response {
            response_type,
            data,
        }
    }
}
