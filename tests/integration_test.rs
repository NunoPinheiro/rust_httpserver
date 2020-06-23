use web_server::http::http_server::HttpServer;
use std::thread;
use std::sync::Arc;
#[test]
fn test(){

    let server =  HttpServer::new("127.0.0.1", 7878);
    //thread::spawn(||);
    server.listen()
}