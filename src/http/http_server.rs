use crate::http::http_router::HttpRouter;
use crate::http::{HttpMethod, HttpRequest, HttpResponse, HttpVersion};
use std::collections::HashMap;
use std::io::{BufRead, Read};
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub struct HttpServer{
    listen_addr: &'static str,
    port: u16,
    router: HttpRouter,
}

impl HttpServer {
    pub fn listen(self) {
        let complete_listen_addr = format!("{}:{}", self.listen_addr, self.port);
        let listener = TcpListener::bind(complete_listen_addr.as_str()).unwrap();
        let self_ref = Arc::new(self);
        println!("Listening on {}", complete_listen_addr);
        for stream in listener.incoming() {
            let local_ref = self_ref.clone();
            thread::spawn(move || {
                //println!("Connection established!");
                local_ref.process_message(stream.unwrap());
            });
        }
    }

    pub fn new(listen_addr: &'static str, port: u16) -> HttpServer {
        HttpServer {
            listen_addr,
            port,
            router: HttpRouter::default(),
        }
    }

    pub fn get<T: Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static>(
        &mut self,
        path: &str,
        handler: T,
    ) {
        self.router.on(HttpMethod::GET, path, Arc::new(handler));
    }

    pub fn post<T: Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static>(
        &mut self,
        path: &str,
        handler: T,
    ) {
        self.router.on(HttpMethod::POST, path, Arc::new(handler));
    }

    pub fn put<T: Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static>(
        &mut self,
        path: &str,
        handler: T,
    ) {
        self.router.on(HttpMethod::PUT, path, Arc::new(handler));
    }

    pub fn delete<T: Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static>(
        &mut self,
        path: &str,
        handler: T,
    ) {
        self.router.on(HttpMethod::DELETE, path, Arc::new(handler));
    }

    fn process_message(& self, mut stream: TcpStream) {
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        //println!("First line: {}", line)
        let splits: Vec<&str> = line.split(' ').collect();
        if splits.len() != 3 {
            eprintln!(
                "First line of request had wrong splits size {}",
                splits.len()
            );
            return;
        }

        //TODO ignoring protocol version for now

        let http_version = splits[2].trim();
        let mut http_request = HttpRequest {
            method: HttpMethod::from_method_string(splits[0]),
            path: String::from(splits[1]),
            http_version: HttpVersion::from_str(http_version),
            headers: HashMap::new(),
            content: None,
            route_params: HashMap::new(),
        };

        let mut content_length: Option<usize> = None;
        //Read headers until we find a new line
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            if line == "\r\n" {
                break;
            }

            let splits: Vec<&str> = line.split(": ").collect();
            if splits.len() != 2 {
                eprintln!("Header had wrong splits size {}", line.len());
                return;
            }
            let key = String::from(splits[0]);
            let val = trim(splits[1]);

            if key == "Content-Length" {
                content_length = Some(val.parse().unwrap())
            }

            http_request.headers.insert(key, val);
        }

        if let Some(size) = content_length {
            let mut buffer: Vec<u8> = vec![0; size];
            reader.read_exact(buffer.as_mut_slice()).unwrap();
            http_request.content = Some(buffer);
        }

        let response = self.router.handle(http_request);
        let mut response_builder = String::new();
        response_builder.push_str(
            format!(
                "{} {} {}\r\n",
                http_version,
                response.status_code.to_code(),
                response.status_code.to_string()
            )
            .as_str(),
        );
        stream.write_all(response_builder.as_bytes()).unwrap();

        for header in &response.headers {
            stream
                .write_all(format!("{}: {}\r\n", header.0, header.1).as_bytes())
                .unwrap();
        }
        if let Some(content) = response.content.as_deref() {
            stream
                .write_all(format!("Content-Length: {}\r\n", content.len()).as_bytes())
                .unwrap();
        }
        stream.write_all(b"\r\n").unwrap();

        if let Some(content_bytes) = response.content.as_deref() {
            stream.write_all(content_bytes).unwrap();
        }
    }
}

fn trim(original: &str) -> String {
    let end = original.len() - 2;
    String::from(&original[..end])
}
