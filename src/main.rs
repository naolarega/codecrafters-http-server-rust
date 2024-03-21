use std::{io::{Read, Write}, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut start_line = [u8::default(); 1024];
                stream.read(&mut start_line).unwrap();

                let mut start_line_sections = start_line.split(|byte| byte == &(b' '));

                if let Some(_method) = start_line_sections.next() {
                    /* Handle http verb */
                }

                match start_line_sections.next() {
                    Some(b"/") => { stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap(); },
                    Some(path) => {
                        let mut path_sections = path.split(|byte| byte == &(b'/'));

                        // First empty section
                        path_sections.next().unwrap();

                        let mut response_body = None;

                        if let Some(b"echo") = path_sections.next() {
                            if let Some(random_string) = path_sections.next() {
                                response_body = Some(random_string);
                            }
                        }
                        
                        if let Some(body) = response_body {
                            stream.write(b"HTTP/1.1 200 OK\r\n").unwrap();
                            stream.write(b"Content-Type: text/plain\r\n\r\n").unwrap();
                            stream.write(body).unwrap();
                        } else {
                            stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                        }
                    },
                    _ => return
                }

                stream.flush().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
