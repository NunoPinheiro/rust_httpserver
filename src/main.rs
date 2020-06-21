use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::io::{BufRead, Read};
use  std::io::BufReader;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
        process_message(stream);
    }
}

#[derive(Debug)]
enum HttpMethod{
    GET,
    PUT,
    POST,
    DELETE,
    OPTION
}

#[derive(Debug)]
struct HttpRequest{
    method: HttpMethod,
    path: String,
    headers: HashMap<String, String>, //TODO ignoring multiple headers for the same string for now
    content: Option<Vec<u8>>,
}

impl HttpMethod{
    fn fromMethodString(value: &str) -> HttpMethod{
        match value{
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "OPTION" => HttpMethod::OPTION,
            other => panic!("Unable to find method for '{}'", other)
        }
    }
}

fn process_message(stream: TcpStream){
    let mut stream = BufReader::new(stream);
    let mut line = String::new();
    stream.read_line(&mut line);

    //println!("First line: {}", line)
    let splits: Vec<&str> = line.split(" ").collect();
    if splits.len() != 3{
        eprintln!("First line of request had wrong splits size {}", splits.len());
        return;
    }

    //TODO ignoring protocol version for now

    let mut http_request = HttpRequest{
        method: HttpMethod::fromMethodString(splits[0]),
        path: String::from(splits[1]),
        headers: HashMap::new(),
        content: None
    };

    let mut content_length: Option<usize> = None;
    //Read headers until we find a new line
    loop {
        let mut line = String::new();
        stream.read_line(&mut line);

        if line.eq("\r\n"){
            break;
        }

        let splits: Vec<&str> = line.split(": ").collect();
        if splits.len() != 2{
            eprintln!("Header had wrong splits size {}", line.len());
            return;
        }
        let key = String::from(splits[0]);
        let val = String::from(trim(splits[1]));

        if key == "Content-Length"{
            content_length = Some(val.parse().unwrap())
        }

        http_request.headers.insert(key, val);
    }

    if let Some(size) = content_length {
        let mut buffer: Vec<u8> =  vec![0; size];
        stream.read_exact(buffer.as_mut_slice()).unwrap();
        http_request.content = Some(buffer);
    }

    print_request(http_request);
}

fn trim(original: &str) -> String{
    let end = original.len() - 2;
    String::from(&original[..end])
}

fn print_request(http_request: HttpRequest){
    println!("{:?}", http_request)
    /*
    println!("Method: {}", http_request.method);
    println!("Path: {}", http_request.path);
    for (key, val) in http_request.headers{
        println!("Header {}: {}", key, val);
    }
    */
}