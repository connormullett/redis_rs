//! # RedisRs
//! A simple redis client library
//! This library revolves around the Connection struct.
//! Every request is sent via Connection methods.
//! Requests can also be sent using the `send_raw_request` function.

//! Examples
//! Create a connection and send requests
//!```
//!extern crate redis_rs;
//!use redis_rs::connection::Connection;
//!use redis_rs::response::Response;
//!
//!let host = String::from("127.0.0.1");
//!let client = Connection::new(host, 6379);
//!// send a request
//!let _ = client.send_raw_request("SET FOO BAR".to_string());
//!// or use a supported command
//!let response = client.get("FOO").unwrap();
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
