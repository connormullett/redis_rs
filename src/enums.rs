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
