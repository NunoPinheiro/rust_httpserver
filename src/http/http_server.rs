use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::io::{BufRead, Read};
use  std::io::BufReader;
use crate::http::{HttpRequest, HttpMethod};
use crate::http::http_router::{HttpRouter, HttpRouteHandler};

pub struct HttpServer<'a>{
    listen_addr: &'a str,
    port: u16,
    router: HttpRouter
}

impl<'a> HttpServer<'a>{
    pub fn listen(self){
        let listener = TcpListener::bind(format!("{}:{}", self.listen_addr, self.port)).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            println!("Connection established!");
            process_message(stream);
        }
    }

    pub fn new(listen_addr: &str, port: u16) -> HttpServer{
        HttpServer{
            listen_addr,
            port,
            router: HttpRouter::new()
        }
    }

    pub fn get(&mut self, path: &str, handler: Box<HttpRouteHandler>){
        self.router.on(HttpMethod::GET, path, handler);
    }

    pub fn post(&mut self, path: &str, handler: Box<HttpRouteHandler>){
        self.router.on(HttpMethod::POST, path, handler);
    }

    pub fn put(&mut self, path: &str, handler: Box<HttpRouteHandler>){
        self.router.on(HttpMethod::PUT, path, handler);
    }


    pub fn delete(&mut self, path: &str, handler: Box<HttpRouteHandler>){
        self.router.on(HttpMethod::DELETE, path, handler);
    }

}

fn process_message(stream: TcpStream){
    let mut stream = BufReader::new(stream);
    let mut line = String::new();
    stream.read_line(&mut line).unwrap();

    //println!("First line: {}", line)
    let splits: Vec<&str> = line.split(" ").collect();
    if splits.len() != 3{
        eprintln!("First line of request had wrong splits size {}", splits.len());
        return;
    }

    //TODO ignoring protocol version for now

    let mut http_request = HttpRequest{
        method: HttpMethod::from_method_string(splits[0]),
        path: String::from(splits[1]),
        headers: HashMap::new(),
        content: None,
        route_params: HashMap::new()
    };

    let mut content_length: Option<usize> = None;
    //Read headers until we find a new line
    loop {
        let mut line = String::new();
        stream.read_line(&mut line).unwrap();

        if line == "\r\n" {
            break;
        }

        let splits: Vec<&str> = line.split(": ").collect();
        if splits.len() != 2{
            eprintln!("Header had wrong splits size {}", line.len());
            return;
        }
        let key = String::from(splits[0]);
        let val = String::from(trim(splits[1]));

        if key == "Content-Length"{
            content_length = Some(val.parse().unwrap())
        }

        http_request.headers.insert(key, val);
    }

    if let Some(size) = content_length {
        let mut buffer: Vec<u8> =  vec![0; size];
        stream.read_exact(buffer.as_mut_slice()).unwrap();
        http_request.content = Some(buffer);
    }

    print_request(http_request);
}

fn trim(original: &str) -> String{
    let end = original.len() - 2;
    String::from(&original[..end])
}

fn print_request(http_request: HttpRequest){
    println!("{:?}", http_request)
}