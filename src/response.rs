use crate::enums::ResponseType;

#[derive(Debug)]
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
