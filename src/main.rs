use web_server::http::http_server::HttpServer;

fn main() {
    let mut server = HttpServer::new("127.0.0.1", 7878);

    server.get("/path", |_| println!("Called"));

    server.get("/test/?param", |x| {
        println!(
            "Called with param: {}",
            x.route_params.get("param").unwrap()
        )
    });
    server.listen()
}
