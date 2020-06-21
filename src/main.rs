use http::http_server::HttpServer;

mod http;

fn main() {
    let server = HttpServer::new("127.0.0.1", 7878);
    server.listen();

}
