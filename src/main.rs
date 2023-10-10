use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buf = [0; 512];
                _stream.read(&mut buf).unwrap();
                let path = handle(&mut buf);
                if path == "/" {
                    _stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                } else {
                    _stream.write_all(b"HTTP/1.1 404 NotFound\r\n\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle(buf: &mut [u8]) -> String {
    let data = String::from_utf8(buf.to_vec()).unwrap();
    let first_line = data.split_once("\r\n\r\n").unwrap().0;
    return first_line.split(" ").nth(1).unwrap().to_string();
}