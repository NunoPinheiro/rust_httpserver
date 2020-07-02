use crate::http::{HttpMethod, HttpRequest, HttpResponse};
use enum_iterator::IntoEnumIterator;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::sync::Arc;

//TODO what should we have here? Should http request handle a drop so we know when it goes out of context we should write the result?
pub type HttpRouteHandler = dyn Fn(HttpRequest) -> HttpResponse + Send + Sync;

type Routes = HashMap<String, HttpRouteNode>;

trait RouteAdd {
    fn on(&mut self, original_path: &str, path: &[&str], handler: Arc<HttpRouteHandler>);
}
impl RouteAdd for Routes {
    fn on(&mut self, original_path: &str, path: &[&str], handler: Arc<HttpRouteHandler>) {
        let child_key = String::from(path[0]);
        let child = self
            .entry(child_key)
            .or_insert(HttpRouteNode::new())
            .borrow_mut();
        child.on(original_path, path, handler);
    }
}

struct HttpRouteNode {
    handler: Option<Arc<HttpRouteHandler>>,
    children: Routes,
    var_name: Option<String>,
    wildcard: bool,
}

impl HttpRouteNode {
    fn new() -> HttpRouteNode {
        HttpRouteNode {
            handler: None,
            children: HashMap::new(),
            var_name: None,
            wildcard: false,
        }
    }

    pub fn on(&mut self, original_path: &str, path: &[&str], handler: Arc<HttpRouteHandler>) {
        let self_path_part = path[0];
        if self_path_part.starts_with("?") {
            self.var_name = Some(String::from(&self_path_part[1..]));
        }
        if self_path_part == "*" {
            self.wildcard = true;
            if path.len() > 1 {
                panic!(
                    "Last '*' for path part that is not the last: {}",
                    original_path
                )
            }
        }
        if path.len() > 1 {
            self.children.on(original_path, &path[1..], handler);
        } else {
            self.handler = Some(handler);
        }
    }
}

pub struct HttpRouter {
    roots: HashMap<HttpMethod, Routes>,
    not_found_handler: Option<Arc<HttpRouteHandler>>,
}

impl HttpRouter {
    pub fn new() -> HttpRouter {
        let mut roots = HashMap::new();

        for method in HttpMethod::into_enum_iter() {
            roots.insert(method, HashMap::new());
        }

        HttpRouter {
            roots,
            not_found_handler: None,
        }
    }

    pub fn on(&mut self, method: HttpMethod, mut path: &str, handler: Arc<HttpRouteHandler>) {
        if path.starts_with("/") {
            path = &path[1..];
        }
        let parts: Vec<&str> = path.split("/").collect();
        let routes = self.roots.get_mut(&method).unwrap();
        routes.on(path, parts.borrow(), handler);
    }

    pub fn handle(&self, mut http_request: HttpRequest) -> HttpResponse {
        let mut routes = self.roots.get(http_request.method.borrow()).unwrap();
        let mut node: Option<&HttpRouteNode> = None;
        let mut path = http_request.path.as_str();
        if path.starts_with("/") {
            path = &path[1..];
        }
        for part in path.split("/") {
            if let Some(inner_node) = routes.get(part) {
                node = Some(inner_node);
                routes = &inner_node.children;
            } else if let Some(inner_node) = routes
                .iter()
                .map(|x| x.1)
                .find(|x| x.wildcard == true || x.var_name.is_some())
            {
                if let Some(var_name) = &inner_node.var_name {
                    http_request
                        .route_params
                        .insert(var_name.clone(), String::from(part));
                }
                node = Some(inner_node);
                if inner_node.wildcard {
                    break;
                }
            } else {
                return self.handle_not_found(http_request);
            }
        }

        match node {
            Some(node) if node.handler.is_some() => (node.handler.as_ref()).unwrap()(http_request),
            _ => self.handle_not_found(http_request),
        }
    }

    pub fn handle_not_found(&self, http_request: HttpRequest) -> HttpResponse {
        if let Some(not_found_handler) = &self.not_found_handler {
            not_found_handler(http_request)
        } else {
            HttpResponse::new().not_found()
        }
    }

    pub fn on_not_found(&mut self, not_found_handler: Arc<HttpRouteHandler>) {
        self.not_found_handler = Some(not_found_handler);
    }
}

#[cfg(test)]
mod tests {
    use crate::http::http_router::HttpRouter;
    use crate::http::{HttpMethod, HttpRequest, HttpResponse, HttpVersion, StatusCode};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn test_http_request(method: HttpMethod, path: &str) -> HttpRequest {
        HttpRequest {
            method,
            http_version: HttpVersion::_1_1,
            path: String::from(path),
            headers: HashMap::new(),
            content: None,
            route_params: HashMap::new(),
        }
    }

    #[test]
    fn it_calls_not_found_handler() {
        let mut router = HttpRouter::new();
        let on_not_found = |_| {
            HttpResponse::new()
                .with_string_content("Not Found!")
                .not_found()
        };
        router.on_not_found(Arc::new(on_not_found));
        let response = router.handle(test_http_request(HttpMethod::GET, "/non/existent"));
        assert_eq!(response.content_as_string(), "Not Found!");
    }

    #[test]
    fn it_calls_default_not_found_handler() {
        let router = HttpRouter::new();
        let response = router.handle(test_http_request(HttpMethod::GET, "/non/existent"));
        assert_eq!(response.status_code, StatusCode::_404);
    }

    #[test]
    fn it_calls_route_handler() {
        let mut router = HttpRouter::new();
        let on_handler = |_| HttpResponse::new().with_string_content("Called!");
        router.on(HttpMethod::GET, "/hello", Arc::new(on_handler));
        let response = router.handle(test_http_request(HttpMethod::GET, "/hello"));
        assert_eq!(response.content_as_string(), "Called!");
    }

    #[test]
    fn it_calls_deeper_route_handler() {
        let mut router = HttpRouter::new();
        let on_handler = |_| HttpResponse::new().with_string_content("Called!");
        router.on(HttpMethod::GET, "/hello/world", Arc::new(on_handler));
        let response = router.handle(test_http_request(HttpMethod::GET, "/hello/world"));
        assert_eq!(response.content_as_string(), "Called!");
    }

    #[test]
    fn it_calls_route_handler_wild_card() {
        let mut router = HttpRouter::new();
        let on_handler = |x: HttpRequest| HttpResponse::new().with_string_content(x.path.as_str());
        router.on(HttpMethod::GET, "/static/*", Arc::new(on_handler));
        let response = router.handle(test_http_request(HttpMethod::GET, "/static/path/for/file"));
        assert_eq!(response.content_as_string(), "/static/path/for/file");
    }

    #[test]
    fn it_calls_route_with_right_value() {
        let mut router = HttpRouter::new();
        let on_handler = |x: HttpRequest| {
            HttpResponse::new().with_string_content(x.route_params.get("key").unwrap())
        };
        router.on(HttpMethod::GET, "/with_var/?key", Arc::new(on_handler));
        let response = router.handle(test_http_request(HttpMethod::GET, "/with_var/expected"));
        assert_eq!(response.content_as_string(), "expected");
    }

    #[test]
    fn it_calls_route_handler_for_root() {
        let mut router = HttpRouter::new();
        let on_handler = |_| HttpResponse::new().with_string_content("Called for root!");
        router.on(HttpMethod::GET, "/", Arc::new(on_handler));
        let response = router.handle(test_http_request(HttpMethod::GET, "/"));
        assert_eq!(response.status_code, StatusCode::_200);
        assert_eq!(response.content_as_string(), "Called for root!");
    }

    #[test]
    fn it_calls_route_for_right_method() {
        let mut router = HttpRouter::new();
        let on_handler = |_| HttpResponse::new().with_string_content("Called!");
        router.on(HttpMethod::POST, "/path", Arc::new(on_handler));
        let response = router.handle(test_http_request(HttpMethod::GET, "/path"));
        assert_eq!(response.status_code, StatusCode::_404);

        let response = router.handle(test_http_request(HttpMethod::POST, "/path"));
        assert_eq!(response.status_code, StatusCode::_200);
    }
}
