//! # RedisRs
//! A simple redis client library
//! This library revolves around the Connection struct.
//! Every request is sent via Connection methods.
//! Requests can also be sent using the `send_raw_request` function.

//! Examples
//! Create a connection and send requests
//!```
//!let client = Connection::new("127.0.0.1", 6379);
//!// send a request
//!let _ = client.send_raw_request("SET FOO BAR");
//!// or use a supported command
//!let response = client.get("FOO");
//!
//!// match against the response to extract the value
//!if let Response::BulkString(value) = response {
//!  println!("{}", value);
//!}
//!```

pub mod connection;
pub mod enums;
mod parse;
pub mod response;
