use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use crate::heap_buffer::get_char_buffer;

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; 0x400];
    let _ = stream.read(&mut buffer).await;
    let body = get_char_buffer();
    let header = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/xml; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        body.as_bytes().len()
    );
    let _ = stream.write_all(header.as_bytes()).await;
    let _ = stream.write_all(body.as_bytes()).await;
    let _ = stream.flush().await;
}

pub async fn run_sitemap_server(addr: &str) {
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|e| {
        panic!("Failed to bind {}: {}", addr, e);
    });
    println!("Sitemap server listening on http://{}", addr);
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    handle_connection(stream).await;
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
