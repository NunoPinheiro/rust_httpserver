use enum_iterator::IntoEnumIterator;
use std::collections::HashMap;

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

pub enum HttpContentType {
    TEXTPLAIN,
}

pub enum HttpVersion {
    _1_1,
}

impl HttpVersion {
    fn from_str(val: &str) -> HttpVersion {
        match val {
            "HTTP/1.1" => HttpVersion::_1_1,
            _ => panic!("Http version not supported: {}", val),
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            HttpVersion::_1_1 => "HTTP/1.1",
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum StatusCode {
    _404,
    _200,
}

impl StatusCode {
    pub fn to_string(&self) -> &str {
        match self {
            StatusCode::_404 => "Not Found",
            StatusCode::_200 => "OK",
        }
    }

    pub fn to_code(&self) -> u16 {
        match self {
            StatusCode::_404 => 404,
            StatusCode::_200 => 200,
        }
    }
}

//TODO get better API to write a response
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub content_type: Option<HttpContentType>,
    pub content: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn new() -> HttpResponse {
        HttpResponse {
            status_code: StatusCode::_200,
            content_type: None,
            content: None,
        }
    }

    pub fn with_string_content(mut self, content: &str) -> HttpResponse {
        self.content_type = Some(HttpContentType::TEXTPLAIN);
        self.content = Some(content.as_bytes().to_vec());
        self
    }

    pub fn ok(mut self) -> HttpResponse {
        self.status_code = StatusCode::_200;
        self
    }

    pub fn not_found(mut self) -> HttpResponse {
        self.status_code = StatusCode::_404;
        self
    }

    //Explicitly convert the data so we don't need to re-allocate memory
    pub fn content_as_string(self) -> String {
        String::from_utf8(self.content.unwrap()).unwrap()
    }
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub http_version: HttpVersion,
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
