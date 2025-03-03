use serde_json::Map;
use serde_json::Value;

use crate::modules::filetype::FileType;
use crate::modules::http_request::HttpMethod;
use crate::modules::http_request::HttpRequest;
use crate::modules::schemas::Schwema;
use crate::tcp_server::TcpServer;
use crate::traits::Server;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Clone)]
pub struct HttpServer {
    server: TcpServer,
    headers: HashMap<String, String>,
}

impl HttpServer {
    pub fn new(host: &str, port: u16) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Server".to_string(), "Crude Server".to_string());
        headers.insert("Content-Type".to_string(), "text/html".to_string());

        Self {
            server: TcpServer::new(host, port),
            headers,
        }
    }

    fn response_line(&self, status_code: i32) -> &str {
        // TODO: Make this an enum.
        match status_code {
            200 => "HTTP/1.1 200 OK",
            404 => "HTTP/1.1 404 Not Found",
            501 => "HTTP/1.1 501 Not implemented",
            403 => "HTTP/1.1 403 Forbidden",
            _ => "HTTP/1.1 500 Internal Server Error", // Add a fallback for unexpected codes.
        }
    }

    fn response_headers(
        &self,
        extra_headers: Option<HashMap<String, String>>,
    ) -> HashMap<String, String> {
        let mut headers = self.headers.clone();

        if let Some(extra) = extra_headers {
            println!("{:?}", extra);
            extra.into_iter().for_each(|(key, value)| {
                headers.insert(key, value);
            });
        }

        println!("{:?}", headers);

        headers
    }

    // POST /test HTTP/1.1
    // Host: example.com
    // Content-Type: application/x-www-form-urlencoded
    // Content-Length: 27
    // field1=value1&field2=value2

    // pub fn handle_post(&self, request: HttpRequest) {
    //     let path = Path::new("database/seeqwel.txt");
    //     let file = File::open(path).expect("Could not open file");
    // }
    //

    // Refactor the match code to be a separate function

    pub fn handle_get(&self, request: HttpRequest) -> Vec<u8> {
        let filename = request
            .uri
            .unwrap()
            .strip_prefix("/")
            .unwrap_or("")
            .to_owned();

        let extension = filename.rsplit('.').next().expect("Extension not provided");
        let file_path = format!("static_assets/{}", filename);

        let mut file = FileType::new(extension, file_path);
        let extra_headers = file.mimetype_to_hashmap();

        let content = match file.read_file() {
            Some(data) => data, // Successfully read file
            None => {
                // 404 Response
                let response = format!(
                    "{}\r\n{}\r\n{}\r\n{}",
                    self.response_line(404),
                    self.response_headers(None)
                        .into_iter()
                        .map(|(key, value)| format!("{}: {}", key, value))
                        .collect::<Vec<String>>()
                        .join("\r\n"),
                    "\r\n",
                    "<h1>Resource Not Found</h1>"
                );
                return response.into_bytes();
            }
        };

        // 200 Response
        let response_headers = {
            let mut headers = self
                .response_headers(extra_headers)
                .into_iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<String>>();
            headers.push(format!("Content-Length: {}", content.len()));
            headers.join("\r\n")
        };

        let mut response = Vec::new();
        response.extend_from_slice(self.response_line(200).as_bytes());
        response.extend_from_slice(response_headers.as_bytes());
        response.extend_from_slice(b"\r\n\r\n");
        response.extend_from_slice(&content);

        response
    }

    // fn interpret_request_headers()

    fn handle_post(&self, request: HttpRequest) -> Vec<u8> {
        let filename = request
            .uri
            .unwrap()
            .strip_prefix("/")
            .unwrap_or("")
            .to_owned();

        // let extension = filename.rsplit('.').next().unwrap_or("txt");
        let file_path = format!("database/{}", filename);
        let extra_headers = None; // Might change this later
        let response_line: &[u8];
        let response_body: &[u8];
        let path = Path::new(&file_path);
        let display = path.display();

        if path.is_file() {
            response_line = self.response_line(403).as_bytes();
            response_body =
                "{'message': 'Forbidden creation request for an existing resource'}".as_bytes();
        } else {
            response_line = self.response_line(200).as_bytes();
            let mut file = match File::create(path) {
                Err(why) => panic!("Something went wrong while creating the file: {}", why),
                Ok(file) => file,
            };

            match file.write_all(request.request_body.as_bytes()) {
                Err(why) => panic!("Failed to write to resource: {}", why),
                Ok(_) => {
                    println!("successfully wrote to {}", display);
                }
            }
            response_body = "{'message': 'Resource created successfully'}".as_bytes();
        }
        let response_headers = {
            let mut headers = self
                .response_headers(extra_headers)
                .into_iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<String>>();
            headers.push(format!("Content-Length: {}", response_body.len()));
            headers.join("\r\n")
        };

        let mut response = Vec::new();
        response.extend_from_slice(response_line);
        response.extend_from_slice(response_headers.as_bytes());
        response.extend_from_slice(b"\r\n\r\n");
        response.extend_from_slice(&response_body);

        response
    }
    fn handle_patch(&self, request: HttpRequest) -> Vec<u8> {
        "HTTP/1.1 200 WIP\r\nContent-Type: application/json\r\nContent-Length: 132\r\nLocation: https://api.example.com/resource/12345\r\nDate: Sat, 16 Dec 2024 00:00:00 GMT\r\nConnection: keep-alive\r\n\r\nsay=hi&to=mom"
            .as_bytes()
            .to_vec()
    }

    // Edits existing reesources and creates new ones if they don't exist.
    // TODO:
    // Use the schwema to make edit easier, right now you aren't editing a schema viz. not the correct approach. Fix that.
    // Fix the headers; Add more to them: Content location, Content type.
    fn handle_put(&self, request: HttpRequest) -> Vec<u8> {
        // Extract the filename from the URI
        let filename = request
            .uri
            .unwrap()
            .strip_prefix("/")
            .unwrap_or("")
            .to_owned();
        let file_path = format!("database/{}", filename);
        let path = Path::new(&file_path);

        // Initialize content as an empty JSON object
        let mut content: Value = Value::Object(Map::new());

        // Parse the request body into a JSON value
        let parsed_request_body: Value =
            serde_json::from_str(request.request_body.as_str()).expect("Failed to parse JSON");

        // Check if the file exists
        if path.is_file() {
            // Read the file's content and parse it as JSON
            let file_content = fs::read_to_string(path).expect("Failed to read file");
            content = serde_json::from_str(&file_content).expect("Failed to parse existing JSON");
        }

        // Merge the parsed request body into the content
        if let Some(obj) = parsed_request_body.as_object() {
            if let Some(existing_obj) = content.as_object_mut() {
                for (key, value) in obj.iter() {
                    existing_obj.insert(key.clone(), value.clone());
                }
            }
        }

        // Write the updated JSON content back to the file
        let mut file = File::create(path).expect("Failed to create or overwrite the file");
        let updated_content =
            serde_json::to_string_pretty(&content).expect("Failed to serialize JSON");
        file.write_all(updated_content.as_bytes())
            .expect("Failed to write to file");

        // Prepare and send the response
        let response_line = self.response_line(200).as_bytes();
        let response_body = "{'message': 'Resource updated successfully'}".as_bytes();
        let response_headers = self
            .response_headers(None)
            .into_iter()
            .map(|(key, value)| format!("{}:{}", key, value).to_string())
            .collect::<Vec<String>>()
            .join("\r\n");
        let response = [
            response_line,
            response_headers.as_bytes(),
            "\r\n\r\n".as_bytes(),
            response_body,
        ]
        .concat();

        response
    }

    fn handle_delete(&self, request: HttpRequest) -> Vec<u8> {
        let filename = request
            .uri
            .unwrap()
            .strip_prefix("/")
            .unwrap_or("")
            .to_owned();

        // let extension = filename.rsplit('.').next().unwrap_or("txt");
        let file_path = format!("database/{}", filename);
        let extra_headers = None; // Might change this later
        let response_line: &[u8];
        let response_body: &[u8];
        let mut response_headers: Vec<String>;
        let path = Path::new(&file_path);

        match fs::remove_file(path) {
            Err(why) => {
                response_line = self.response_line(404).as_bytes();
                response_body = "{'message': '404 file not found!'}".as_bytes();
                response_headers = self
                    .response_headers(extra_headers)
                    .into_iter()
                    .map(|(key, value)| format!("{}: {}", key, value))
                    .collect::<Vec<String>>();
                response_headers.push(format!("Content-Length: {}", response_body.len()));
                panic!("Failed to delete resource: {}", why)
            }
            Ok(_) => {
                response_line = self.response_line(200).as_bytes();
                response_body = "{'message': 'Resource deletion successful!'}".as_bytes();
                response_headers = self
                    .response_headers(extra_headers)
                    .into_iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect::<Vec<String>>();
                response_headers.push(format!("Content-Length: {}", response_body.len()));
                println!("successfully Deleted {}", filename);
            }
        }

        let mut response = Vec::new();
        response.extend_from_slice(response_line);
        response.extend_from_slice(response_headers.join("\r\n").as_bytes());
        response.extend_from_slice(b"\r\n\r\n");
        response.extend_from_slice(&response_body);

        response
    }

    pub fn http_501_handler(&self, request: HttpRequest) -> Vec<u8> {
        let response_line = self.response_line(501);
        let response_headers: Vec<String> = self
            .response_headers(None)
            .into_iter()
            .map(|(key, value)| format!("{}:{}", key, value))
            .collect();
        let blank_line = "\r\n";
        let response_body = "<h1>501 Not Implemented</h1>";

        let response = format!(
            "{}\r\n{}\r\n{}{}",
            response_line,
            response_headers.join("\r\n"),
            blank_line,
            response_body
        );

        response.as_bytes().to_vec()
    }
}

impl Server for HttpServer {
    fn handle_request(&self, data: &[u8]) -> Vec<u8> {
        let request = HttpRequest::new(data);

        let response = match request.method {
            HttpMethod::GET => self.handle_get(request),
            HttpMethod::POST => self.handle_post(request),
            HttpMethod::PATCH => self.handle_patch(request),
            HttpMethod::PUT => self.handle_put(request),
            HttpMethod::DELETE => self.handle_delete(request),
            _ => self.http_501_handler(request),
        };

        print!("{:?}", response);

        response

        // let headers: Vec<String> = self
        //     .response_headers(None)
        //     .into_iter()
        //     .map(|(key, value)| format!("{}: {}", key, value))
        //     .collect();

        // let header_string = headers.join("\r\n");
        // let blank_line = "\r\n";
        // let response_body = "
        //     <html>
        //     <body>
        //     <h1>request received!</h1>
        //     </body>
        //     </html>
        //     ";

        // let response = format!(
        //     "{}\r\n{}\r\n{}{}",
        //     response_line, header_string, blank_line, response_body
        // );
        // response.as_bytes().to_vec()
    }

    fn start(&self) {
        let handler = Arc::new(self.clone());
        self.server.serve(handler);
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];

        match stream.read(&mut buffer) {
            Ok(size) => {
                let request_data = &buffer[..size];
                let response_data = self.handle_request(request_data);
                if let Err(e) = stream.write_all(&response_data) {
                    eprintln!("Failed to send response: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to read from connection: {}", e),
        }
    }
}
