use reqwest;
use std::thread;
use web_server::http::http_server::HttpServer;
use web_server::http::HttpResponse;
#[test]
fn test() {
    let mut server = HttpServer::new("127.0.0.1", 7878);

    server.get("/path", |_| {
        HttpResponse::new().with_string_content("Found!")
    });
    thread::spawn(|| server.listen());
    let resp = reqwest::blocking::get("http://localhost:7878/path")
        .unwrap()
        .text()
        .unwrap();
    assert_eq!(resp, "Found!")
}
