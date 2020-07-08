use crate::http::{HttpContentType, HttpRequest, HttpResponse};
use std::fs;

pub struct FileServer {
    base_folder: String,
    base_path: String,
}

//TODO consider adding caching to the files
//TODO add templating support
impl FileServer {
    pub fn new(base_path: String, base_folder: String) -> Self {
        let base_path = if !base_path.starts_with('/') {
            format! {"/{}", base_path}
        } else {
            base_path
        };
        FileServer {
            base_folder,
            base_path,
        }
    }

    pub fn handle(&self, request: HttpRequest) -> HttpResponse {
        let sub_path = &request.path[self.base_path.len()..];
        let file_system_path = format!("{}/{}", self.base_folder, sub_path);
        println!(
            "base: {}, called: {}, part: {}",
            self.base_path, request.path, sub_path
        );
        match fs::read(file_system_path) {
            //TODO how should we handle the content type? Based on file extension with possibility of custom function/hardcoded value?
            //TODO copying the file into memory and further to the buffer might not be the best option
            Ok(result) => {
                HttpResponse::default().with_byte_content(result, HttpContentType::TEXTPLAIN)
            }
            //TODO should handle different kinds of error
            _ => {
                eprintln!("Can't find file!");
                HttpResponse::default().not_found()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::http::file_server::FileServer;
    use crate::http::{HttpMethod, HttpRequest};

    #[test]
    fn it_serves_present_file() {
        let file_server = FileServer::new(String::from("static/content"), String::from("static/"));
        let request = HttpRequest::new(
            HttpMethod::GET,
            String::from("static/content/test_content.txt"),
        );
        let response = file_server.handle(request);

        assert_eq!(response.content_as_string(), "Test content here!\n");
    }
}
