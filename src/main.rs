use std::{
    collections::HashMap,
    env::args,
    fs::{read_dir, File, ReadDir},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use itertools::Itertools;

enum Method {
    GET,
    POST,
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        use Method::*;

        match value {
            "GET" => GET,
            "POST" => POST,
            _ => panic!("unssuported method"),
        }
    }
}

struct Request {
    method: Method,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    pub fn new(stream: &mut TcpStream) -> Self {
        let mut stream_buffer_reader = BufReader::new(stream);

        let mut start_line_string = String::new();

        stream_buffer_reader
            .read_line(&mut start_line_string)
            .unwrap();

        let start_line = start_line_string
            .split(' ')
            .map(|a| a.to_string())
            .collect::<Vec<String>>();

        let mut headers = HashMap::new();

        let mut content_length = 0;

        loop {
            let mut header = String::new();

            stream_buffer_reader.read_line(&mut header).unwrap();

            if &header == "\r\n" {
                break;
            }

            let key_value = header
                .split(':')
                .map(|a| a.to_string())
                .collect::<Vec<String>>();

            if key_value[0].to_lowercase() == "content-length" {
                content_length = key_value[1].parse().unwrap();
            }

            if key_value.len() >= 2 {
                headers.insert(
                    key_value[0].to_lowercase(),
                    key_value[1..].join(":").trim().to_owned(),
                );
            }
        }

        let mut body = [u8::default(), content_length];

        if content_length > 0 {
            stream_buffer_reader.read_exact(&mut body).unwrap();
        }

        Self {
            method: Method::from(start_line[0].as_str()),
            path: start_line[1].to_owned(),
            version: start_line[2].to_owned(),
            headers,
            body: String::from_utf8_lossy(&body).to_string(),
        }
    }
}

enum StatusCode {
    OK = 200,
    NotFound = 404,
}

impl ToString for StatusCode {
    fn to_string(&self) -> String {
        use StatusCode::*;

        match self {
            OK => "200 OK".to_string(),
            NotFound => "404 Not Found".to_string(),
        }
    }
}

struct Response<'a> {
    tcp_stream: &'a TcpStream,
    status_code: Option<StatusCode>,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl<'a> Response<'a> {
    fn new(tcp_stream: &'a mut TcpStream) -> Self {
        Self {
            tcp_stream,
            status_code: None,
            headers: HashMap::new(),
            body: None,
        }
    }

    fn set_status_code(&mut self, status_code: StatusCode) -> &mut Self {
        self.status_code = Some(status_code);
        self
    }

    fn set_header(&mut self, key: &str, value: &str) -> &mut Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    fn send(&mut self, body: Option<String>) -> Result<bool, &str> {
        if let Some(status_code) = &self.status_code {
            self.tcp_stream
                .write(format!("HTTP/1.1 {}\r\n", status_code.to_string()).as_bytes())
                .unwrap();
        } else {
            return Err("status code not set");
        }

        for (key, value) in self.headers.iter() {
            self.tcp_stream
                .write(format!("{}:{}\r\n", key.to_lowercase(), value).as_bytes())
                .unwrap();
        }

        if let Some(body) = body {
            self.tcp_stream
                .write(format!("content-length:{}\r\n", body.len()).as_bytes())
                .unwrap();
            self.tcp_stream.write(b"\r\n").unwrap();

            self.tcp_stream.write(body.as_bytes()).unwrap();
        } else {
            self.tcp_stream.write(b"\r\n").unwrap();
        }

        self.tcp_stream.flush().unwrap();

        Ok(true)
    }
}

fn handle(mut stream: TcpStream) {
    let request = Request::new(&mut stream);
    let mut response = Response::new(&mut stream);

    match request.path.as_str() {
        "/" => {
            response.set_status_code(StatusCode::OK).send(None).unwrap();
        }
        "/user-agent" => {
            if let Some(user_agent) = request.headers.get("user-agent") {
                response
                    .set_status_code(StatusCode::OK)
                    .set_header("Content-Type", "text/plain")
                    .send(Some(user_agent.to_owned()))
                    .unwrap();
            } else {
                response
                    .set_status_code(StatusCode::NotFound)
                    .send(None)
                    .unwrap();
            }
        }
        file_path if file_path.starts_with("/files") => {
            let args = args().collect::<Vec<_>>();

            let mut directory_path = None;

            if let Some((arg_position, _)) = args.iter().find_position(|arg| *arg == "--directory")
            {
                directory_path = Some(args[arg_position + 1].to_owned());
            }

            if let Some(directory_path) = directory_path {
                let dir_entries = read_dir(directory_path).unwrap();

                if let Some(existing_file) = dir_entries.flatten().find(|entry| {
                    entry.file_name() == file_path.strip_prefix("/files/").unwrap_or_default()
                }) {
                    let mut file_content = String::new();

                    File::open(existing_file.path())
                        .unwrap()
                        .read_to_string(&mut file_content)
                        .unwrap();

                    response
                        .set_status_code(StatusCode::OK)
                        .set_header("Content-Type", "application/octet-stream")
                        .send(Some(file_content))
                        .unwrap();
                } else {
                    response
                        .set_status_code(StatusCode::NotFound)
                        .send(None)
                        .unwrap();
                }
            } else {
                response
                    .set_status_code(StatusCode::NotFound)
                    .send(None)
                    .unwrap();
            }
        }
        path => {
            let path_sections = path.split('/').collect::<Vec<&str>>();

            let mut response_body = None;

            if path_sections[1] == "echo" {
                response_body = Some(path_sections[2..].join("/"))
            }

            if let Some(body) = response_body {
                response
                    .set_status_code(StatusCode::OK)
                    .set_header("Content-Type", "text/plain")
                    .send(Some(body))
                    .unwrap();
            } else {
                response
                    .set_status_code(StatusCode::NotFound)
                    .send(None)
                    .unwrap();
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming().flatten() {
        thread::spawn(|| handle(stream));
    }
}
