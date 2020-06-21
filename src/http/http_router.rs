use crate::http::{HttpRequest, HttpMethod};
use std::collections::HashMap;
use enum_iterator::IntoEnumIterator;
use std::borrow::{Borrow, BorrowMut};
use std::env::var;

//TODO what should we have here? Should http request handle a drop so we know when it goes out of context we should write the result?
type HttpRouteHandler = dyn Fn(HttpRequest) -> ();

struct HttpRouteNode{
    handler: Option<Box<HttpRouteHandler>>,
    children: HashMap<String, HttpRouteNode>,
    var_name: Option<String>,
    wildcard: bool
}

impl HttpRouteNode{
    fn new() -> HttpRouteNode{
        HttpRouteNode{
            handler: None,
            children: HashMap::new(),
            var_name: None,
            wildcard: false
        }
    }

    pub fn on(&mut self, original_path: &String, mut path: &[&str], handler: Box<HttpRouteHandler>){
        //TODO should fail if already has inner wildcard node
        //TODO we should have special cases when root is wildcard or var
        match path.len(){
            0 => {
                //Should register handler in current node
                self.handler = Some(handler);
            }
            _ => {
                let key = path[0];
                if key == "*" && path.len() > 1{
                    panic!("Last '*' for path part that is not the last: {}", original_path)
                }
                let mut child =self.children.entry(String::from(key)).or_insert(HttpRouteNode::new()).borrow_mut();
                child.on(original_path, &path[1..], handler)
            }
        }
    }
}

struct HttpRouter{
    roots: HashMap<HttpMethod, HttpRouteNode>,
    not_found_handler: Option<Box<HttpRouteHandler>>,
}

impl HttpRouter{
    pub fn new() -> HttpRouter{
        let mut roots = HashMap::new();

        for method in HttpMethod::into_enum_iter(){
            roots.insert(method, HttpRouteNode::new());
        }

        HttpRouter{
            roots,
            not_found_handler: None
        }
    }

    pub fn on(&mut self, method: HttpMethod, path: String, handler: Box<HttpRouteHandler>){
        let parts: Vec<&str> = path.split("/").collect();
        self.roots.get_mut(&method).unwrap().on(&path,parts.borrow(), handler);
    }

    pub fn handle(&mut self, mut http_request: HttpRequest){
        let mut node = self.roots.get(http_request.method.borrow()).unwrap();
        for part in http_request.path.split("/"){
            if let Some(inner_node) = node.children.get(part){
                node = inner_node;
            }
            else if let Some(inner_node) = node.children.iter().map(|x| x.1).find(|x| x.wildcard == true || x.var_name.is_some()){
                if let Some(var_name) = &inner_node.var_name{
                    http_request.route_params.insert(var_name.clone(), String::from(part));
                }
                node = inner_node;
            }
            else{
                if let Some(not_found_handler) = &self.not_found_handler{
                    not_found_handler(http_request);
                }
                return;
            }
        }

        if let Some(handler) = &node.handler {
            handler(http_request)
        }
        else{
            panic!("Code shouldn't have reached this point")
        }
    }

    pub fn on_not_found(&mut self, not_found_handler: Box<HttpRouteHandler>){
        self.not_found_handler = Some(not_found_handler);
    }
}

#[cfg(test)]
mod tests {
    use crate::http::http_router::{HttpRouter, HttpRouteHandler};
    use crate::http::{HttpRequest, HttpMethod};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::collections::HashMap;

    fn test_http_request(method: HttpMethod, path: &str) -> HttpRequest{
        HttpRequest{
            method,
            path: String::from(path),
            headers: HashMap::new(),
            content: None,
            route_params: HashMap::new()
        }
    }

    #[test]
    fn it_calls_not_found_handler() {
        let mut router = HttpRouter::new();
        let mut not_found = Rc::new(RefCell::new(false));
        let not_found_copy = not_found.clone();
        let on_not_found = move |x: HttpRequest| *not_found_copy.borrow_mut() = true;
        router.on_not_found(Box::new(on_not_found));
        router.handle(test_http_request(HttpMethod::GET, "/non/existent"));
        assert_eq!(*not_found.borrow(), true);
    }


    #[test]
    fn it_calls_route_handler() {
        let mut router = HttpRouter::new();
        let mut handler_called = Rc::new(RefCell::new(false));
        let handler_called_copy = handler_called.clone();
        let on_handler = move |x: HttpRequest| *handler_called_copy.borrow_mut() = true;
        router.on(HttpMethod::GET, String::from("/hello"), Box::new(on_handler));
        router.handle(test_http_request(HttpMethod::GET, "/hello"));
        assert_eq!(*handler_called.borrow(), true);
    }


    #[test]
    fn it_calls_route_handler_wild_card() {
        let mut router = HttpRouter::new();
        let mut handler_called = Rc::new(RefCell::new(false));
        let handler_called_copy = handler_called.clone();
        let on_handler = move |x: HttpRequest| *handler_called_copy.borrow_mut() = true;
        router.on(HttpMethod::GET, String::from("/static/*"), Box::new(on_handler));
        router.handle(test_http_request(HttpMethod::GET, "/static/path/for/file"));
        assert_eq!(*handler_called.borrow(), true);
    }

    #[test]
    fn it_calls_route_with_right_value() {
        let mut router = HttpRouter::new();
        let mut handler_called_result = Rc::new(RefCell::new(String::from("false")));
        let handler_called_copy = handler_called_result.clone();
        let on_handler = move |x: HttpRequest| *handler_called_copy.borrow_mut() = String::from(x.route_params.get("key").unwrap().clone());
        router.on(HttpMethod::GET, String::from("/with_var/?key"), Box::new(on_handler));
        router.handle(test_http_request(HttpMethod::GET, "/with_var/expected"));
        assert_eq!(*handler_called_result.borrow(), "expected");
    }
}