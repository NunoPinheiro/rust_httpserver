use crate::http::file_server::FileServer;
use crate::http::http_router::HttpRouter;
use crate::http::{HttpMethod, HttpRequest, HttpResponse, HttpVersion};
use crossbeam::channel::unbounded;
use crossbeam::channel::Sender;
use std::collections::HashMap;
use std::io;
use std::io::{BufRead, Read};
use std::io::{BufReader, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct HttpServer {
    listen_addr: &'static str,
    port: u16,
    router: HttpRouter,
    threads_count: u8,
    pub should_turn_off: Arc<AtomicBool>,
}

impl HttpServer {
    pub fn listen(self) {
        let complete_listen_addr = format!("{}:{}", self.listen_addr, self.port);
        let listener = TcpListener::bind(complete_listen_addr.as_str()).unwrap();

        let should_turn_off = self.should_turn_off.clone();
        let sender: Sender<TcpStream> = HttpServer::launch_threads(Arc::new(self));
        println!("Listening on {}", complete_listen_addr);
        listener.set_nonblocking(true).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(unwrapped_stream) => sender.send(unwrapped_stream).unwrap(),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5))
                }
                Err(_) => eprintln!("Error opening new connection"),
            };

            if should_turn_off.load(Relaxed) {
                println!("Server shutting down");
                break;
            }
        }
    }

    pub fn setup_signal_handlers(&self) {
        let should_turn_off = self.should_turn_off.clone();
        ctrlc::set_handler(move || {
            println!("Shutting down server due to external signal");
            HttpServer::close(should_turn_off.clone());
        })
        .unwrap();
    }

    pub fn close(should_turn_off: Arc<AtomicBool>) {
        should_turn_off.store(true, Relaxed)
    }

    fn launch_threads(self_ref: Arc<HttpServer>) -> Sender<TcpStream> {
        let (s, r) = unbounded();

        for _ in 0..self_ref.threads_count {
            let local_ref = self_ref.clone();
            let r = r.clone();
            thread::spawn(move || {
                //println!("Connection established!");
                loop {
                    if let Ok(stream) = r.recv_timeout(Duration::from_secs(60 * 60 * 24)) {
                        local_ref.process_message(stream)
                    }
                }
            });
        }
        s
    }

    pub fn new(listen_addr: &'static str, port: u16, threads_count: u8) -> HttpServer {
        HttpServer {
            listen_addr,
            port,
            router: HttpRouter::default(),
            threads_count,
            should_turn_off: Arc::new(AtomicBool::new(false)),
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

    pub fn serve_files(&mut self, path: &str, base_folder: &str) {
        let (append, base_path) = match path {
            path if path.ends_with("/*") => ("", &path[..path.len() - 2]),
            path if path.ends_with('/') => ("", &path[..path.len() - 1]),
            _ => ("/*", path),
        };

        let path = format!("{}{}", path, append);
        let file_server = FileServer::new(String::from(base_path), String::from(base_folder));
        let handler = move |request| file_server.handle(request);
        self.router
            .on(HttpMethod::GET, path.as_str(), Arc::new(handler));
    }

    fn process_message(&self, mut stream: TcpStream) {
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
        stream.shutdown(Shutdown::Both).unwrap();
    }
}

fn trim(original: &str) -> String {
    let end = original.len() - 2;
    String::from(&original[..end])
}
