use std::thread;
use web_server::http::http_server::HttpServer;
#[test]
fn test() {
    let mut server = HttpServer::new("127.0.0.1", 7878);

    server.get("/path", |_| println!("Called"));
    thread::spawn(|| server.listen());
}
