use std::fs;
use std::path::Path;
use tokio::fs::File;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Logs from your program will appear here! Listening on 127.0.0.1:4221...");
    let args: Vec<String> = std::env::args().collect();
    let mut target_dir: Option<String> = None;
    if let Some(index) = args.iter().position(|arg| arg == "--directory") {
        target_dir = Option::from(args[index + 1].clone());
    }
    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    loop {
        let dir = target_dir.clone();
        let (mut stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            println!("accepted new connection");
            let mut buf = [0; 8096];
            let size = stream.read(&mut buf).await?;
            let path = get_path(&mut buf[..size]);
            if path == "/" {
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
            } else {
                let data = get_data(&mut buf[..size]);
                let method = get_method(&mut buf[..size]);
                handle(method, path, data, &mut stream, dir).await?;
            }
            Ok::<(), anyhow::Error>(())
        });
    }
}

fn get_method(buf: &mut [u8]) -> String {
    let data = String::from_utf8(buf.to_vec()).unwrap();
    let first_line = data.split_once("\r\n").unwrap().0;
    return first_line.split(" ").nth(0).unwrap().to_string();
}

fn get_path(buf: &mut [u8]) -> String {
    let data = String::from_utf8(buf.to_vec()).unwrap();
    let first_line = data.split_once("\r\n").unwrap().0;
    return first_line.split(" ").nth(1).unwrap().to_string();
}

fn get_data(buf: &mut [u8]) -> String {
    let data = String::from_utf8(buf.to_vec()).unwrap();
    let rest = data.split_once("\r\n").unwrap().1;
    return rest.to_string();
}

async fn handle(method: String, path: String, data: String, stream: &mut TcpStream, target_dir: Option<String>) -> anyhow::Result<()> {
    let all: Vec<&str> = path.split("/").collect(); // First element is empty
    let segments = &all[1..];
    if segments[0] == "echo" {
        let echo = segments[1..].join("/");
        let resp = format!("\
HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: {}\r\n\
\r\n\
{}\r\n", echo.len(), echo);
        stream.write_all(resp.as_bytes()).await?;
    } else if segments[0] == "user-agent" {
        for line in data.split("\r\n") {
            if line.starts_with("User-Agent: ") {
                let ua = line.split_once("User-Agent: ").unwrap().1;
                let resp = format!("\
HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: {}\r\n\
\r\n\
{}\r\n", ua.len(), ua);
                stream.write_all(resp.as_bytes()).await?;
            }
        };
    } else if segments[0] == "files" {
        let filename = segments[1..].join("/");
        let filepath = &format!("{}/{}", target_dir.unwrap(), filename);
        let path = Path::new(filepath);
        if method == "GET" {
            if path.exists() {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        let resp = format!("\
HTTP/1.1 200 OK\r\n\
Content-Type: application/octet-stream\r\n\
Content-Length: {}\r\n\
\r\n\
{}", content.len(), content);
                        stream.write_all(resp.as_bytes()).await?;
                    }
                    Err(_) => {
                        stream.write_all(b"HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n").await?;
                    }
                }
            } else {
                stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").await?;
            }
        } else if method == "POST" {
            let mut file = File::create(path).await?;
            let content = data.split_once("\r\n\r\n").unwrap().1;
            file.write_all(content.as_bytes()).await?;
            stream.write_all(b"HTTP/1.1 201 OK\r\n\r\n").await?;
        } else {
            stream.write_all(b"HTTP/1.1 405 METHOD NOT ALLOWED\r\n\r\n").await?;
        }
    } else {
        stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").await?;
    }
    Ok(())
}