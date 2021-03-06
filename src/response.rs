use crate::enums::ResponseType;

#[derive(Debug)]
pub struct Response {
    pub response_type: ResponseType,
    pub data: String,
}
