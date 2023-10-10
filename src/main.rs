use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            println!("accepted new connection");
            let mut buf = [0; 512];
            stream.read(&mut buf).await?;
            let path = get_path(&mut buf);
            if path == "/" {
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
            } else {
                let data = get_data(&mut buf);
                handle(path, data, &mut stream).await?;
            }
            Ok::<(), anyhow::Error>(())
        });
    }
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

async fn handle(path: String, data: String, stream: &mut TcpStream) -> anyhow::Result<()> {
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
            stream.write_all(resp.as_bytes()).await?;
        } else {
            stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").await?;
        }
    } else {
        if segments[0] == "user-agent" {
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
        } else {
            stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").await?;
        }
    }
    Ok(())
}