use tokio::{io::{AsyncWriteExt, AsyncBufReadExt, BufReader}, net::TcpSocket};

use std::io;

#[tokio::main]
async fn main() {
    println!("client");

    let addr = "127.0.0.1:8080".parse().unwrap();

    let socket = TcpSocket::new_v4().unwrap();
    let mut stream = socket.connect(addr).await.unwrap();

    println!("connected");

    let (read_half, mut write_half) = stream.split();
    let mut reader = BufReader::new(read_half);

    loop {
        let mut line = String::new();

        println!("enter request...");
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        write_half.write_all(line.as_bytes()).await.unwrap();
        line.clear();

        println!("reading response...");
        reader.read_line(&mut line).await.unwrap();
        println!("{line}");
        line.clear();
    }
    
}
