use web_server::http::http_server::HttpServer;
use web_server::http::HttpResponse;

fn main() {
    let mut server = HttpServer::new("127.0.0.1", 7878);

    server.get("/path", |_| {
        HttpResponse::new().with_string_content("Olá Malin!")
    });

    server.get("/test/?param", |x| {
        let param = x.route_params.get("param").unwrap();
        let content = format!("Called with param: {}", param);
        HttpResponse::new().with_string_content(content.as_str())
    });
    server.listen()
}
