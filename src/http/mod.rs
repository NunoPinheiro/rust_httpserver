use std::collections::HashMap;
use enum_iterator::IntoEnumIterator;

mod http{}
pub mod http_server;
pub mod http_router;


#[derive(Debug, IntoEnumIterator, PartialEq, Eq, Hash)]
enum HttpMethod{
    GET,
    PUT,
    POST,
    DELETE,
    OPTION
}


#[derive(Debug)]
struct HttpRequest{
    method: HttpMethod,
    path: String,
    headers: HashMap<String, String>, //TODO ignoring multiple headers for the same string for now
    content: Option<Vec<u8>>,
    route_params: HashMap<String, String> //route_params are added by the router to the request
}

impl HttpMethod{
    fn fromMethodString(value: &str) -> HttpMethod{
        match value{
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "OPTION" => HttpMethod::OPTION,
            other => panic!("Unable to find method for '{}'", other)
        }
    }
}

