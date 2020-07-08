use reqwest;
use std::{thread, time};
use web_server::http::http_server::HttpServer;
use web_server::http::HttpResponse;

#[test]
fn simple_path_found() {
    let mut server = HttpServer::new("127.0.0.1", 7878, 1);

    server.get("/path", |_| {
        HttpResponse::default().with_string_content("Found!")
    });
    thread::spawn(|| server.listen());
    thread::sleep(time::Duration::from_millis(100));
    let resp = reqwest::blocking::get("http://localhost:7878/path")
        .unwrap()
        .text()
        .unwrap();
    assert_eq!(resp, "Found!")
}

#[test]
fn file_served() {
    let mut server = HttpServer::new("127.0.0.1", 7879, 1);

    server.serve_files("static", "static");
    thread::spawn(|| server.listen());
    thread::sleep(time::Duration::from_millis(100));
    let resp = reqwest::blocking::get("http://localhost:7879/static/test_content.txt")
        .unwrap()
        .text()
        .unwrap();
    assert_eq!(resp, "Test content here!\n")
}