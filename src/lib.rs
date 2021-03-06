//! # RedisRs
//! A simple redis client library
//! This library revolves around the Connection struct.
//! Every request is sent via Connection methods.
//! Requests can also be sent using the `send_raw_request` function.

//! Examples
//! Create a connection and send requests
//!```
//! extern crate redis_rs;
//! use std::net::TcpStream;
//! use redis_rs::connection::Connection;
//! use redis_rs::response::RedisResponse;
//!
//! let host = "127.0.0.1";
//! let port = 6379;
//! let addr = format!("{}:{}", host, port);
//! let stream = TcpStream::connect(addr).unwrap();
//!
//! // stream can be anything that implements read and write
//! let mut client = Connection::new(host, port, stream);
//!
//! // send a request
//! let _ = client.send_raw_request("SET FOO BAR");
//! // or use a supported command
//! let response = client.get("FOO").unwrap();
//!
//! // match against the response to extract the value
//! if let RedisResponse::BulkString(value) = response {
//!   println!("{}", value);
//! }
//!```

pub mod connection;
pub mod enums;
mod parse;
pub mod response;
