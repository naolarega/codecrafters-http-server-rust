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

                if let Some(path) = start_line_sections.next() {
                    let random_string = match path.split(|byte| byte == &(b'/')).nth(2) {
                        Some(some_string) => some_string,
                        None => b""
                    };

                    stream.write(b"HTTP/1.1 200 OK\r\n").unwrap();
                    stream.write(b"Content-Type: text/plain\r\n\r\n").unwrap();
                    stream.write(random_string).unwrap();
                }

                stream.flush().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
