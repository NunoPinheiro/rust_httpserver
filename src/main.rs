use web_server::http::http_server::HttpServer;
use web_server::http::HttpResponse;

fn main() {
    let mut server = HttpServer::new("127.0.0.1", 7878, 32);

    server.get("/ola", |_| {
        HttpResponse::default().with_string_content("Olá Malin!")
    });

    server.get("/", |_| {
        HttpResponse::default().with_string_content("Welcome!")
    });

    server.get("/test/?param", |x| {
        let param = x.route_params.get("param").unwrap();
        let content = format!("Called with param: {}\n", param);
        let header_val = String::from("my val");
        HttpResponse::default()
            .with_string_content(content.as_str())
            .with_header(String::from("Test"), header_val)
    });

    server.serve_files("static", "static");

    server.setup_signal_handlers();

    server.listen()
}
