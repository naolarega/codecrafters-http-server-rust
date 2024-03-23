use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

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
}

impl Request {
    pub fn new(stream: &mut TcpStream) -> Self {
        let mut buf = Vec::new();

        stream.read_to_end(&mut buf).unwrap();

        let request = String::from_utf8(buf).unwrap();
        let request = request.split("\r\n");
        let request = request.map(|a| a.to_string()).collect::<Vec<String>>();

        let start_line_string = request[0].to_owned();

        let start_line = start_line_string
            .split(' ')
            .map(|a| a.to_string())
            .collect::<Vec<String>>();

        let mut headers = HashMap::new();

        for header in request[1..].iter() {
            if header.is_empty() {
                break;
            }

            let key_value = header
                .split(':')
                .map(|a| a.to_string())
                .collect::<Vec<String>>();

            dbg!(&key_value);

            headers.insert(key_value[0].to_owned(), key_value[1].to_owned());
        }

        Self {
            method: Method::from(start_line[0].as_str()),
            path: start_line[1].to_owned(),
            version: start_line[2].to_owned(),
            headers,
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
                .write(format!("HTTP/1.1 {}\r\n", status_code.to_string()).as_bytes()).unwrap();
        } else {
            return Err("status code not set");
        }

        for (key, value) in self.headers.iter() {
            self.tcp_stream
                .write(format!("{}:{}\r\n", key, value).as_bytes()).unwrap();
        }

        self.tcp_stream.write(b"\r\n").unwrap();

        if let Some(body) = body {
            self.tcp_stream.write(body.as_bytes()).unwrap();
        }

        self.tcp_stream.flush().unwrap();

        Ok(true)
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for mut stream in listener.incoming().flatten() {
        let request = Request::new(&mut stream);
        let mut response = Response::new(&mut stream);

        match request.path.as_str() {
            "/" => {
                response.set_status_code(StatusCode::OK);
            }
            "/user-agent" => {}
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
                        .set_header("Content-Length", format!("{}", body.len()).as_str())
                        .send(Some(body)).unwrap();

                    return;
                } else {
                    response.set_status_code(StatusCode::NotFound);
                }
            }
        }

        response.send(None).unwrap();
    }
}
