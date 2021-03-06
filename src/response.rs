use crate::enums::ResponseType;

#[derive(Debug)]
/// Data associated with a redis RESP response
pub struct Response {
    /// The RESP type the response data is
    pub response_type: ResponseType,
    /// A string representation of the response.
    /// Use response_type to dynamically parse to the correct type
    pub data: String,
}
