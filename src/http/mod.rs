use enum_iterator::IntoEnumIterator;
use std::collections::HashMap;

mod http {}
pub mod http_router;
pub mod http_server;

#[derive(Debug, IntoEnumIterator, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    GET,
    PUT,
    POST,
    DELETE,
    OPTION,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>, //TODO ignoring multiple headers for the same string for now
    pub content: Option<Vec<u8>>,
    pub route_params: HashMap<String, String>, //route_params are added by the router to the request
}

impl HttpMethod {
    fn from_method_string(value: &str) -> HttpMethod {
        match value {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "OPTION" => HttpMethod::OPTION,
            other => panic!("Unable to find method for '{}'", other),
        }
    }
}
