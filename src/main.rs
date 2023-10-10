use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buf = [0; 512];
                _stream.read(&mut buf).unwrap();
                let path = get_path(&mut buf);
                if path == "/" {
                    _stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                } else {
                    handle_echo(path, &mut _stream);
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn get_path(buf: &mut [u8]) -> String {
    let data = String::from_utf8(buf.to_vec()).unwrap();
    let first_line = data.split_once("\r\n\r\n").unwrap().0;
    return first_line.split(" ").nth(1).unwrap().to_string();
}

fn handle_echo(path: String, stream: &mut TcpStream) {
    let all: Vec<&str> = path.split("/").collect(); // First element is empty
    let segments = &all[1..];
    if segments.len() >= 2 {
        if segments[0] == "echo" {
            let echo = segments[1..].join("/");
            let resp = format!("\
HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: {}\r\n\
\r\n\
{}\r\n", echo.len(), echo);
            stream.write_all(resp.as_bytes()).unwrap();
            return;
        }
    }
    stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").unwrap();
}