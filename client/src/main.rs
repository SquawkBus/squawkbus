use tokio::{io::{AsyncWriteExt, AsyncBufReadExt, BufReader}, net::TcpSocket};

//use std::io;

#[tokio::main]
async fn main() {
    println!("client");

    let addr = "127.0.0.1:8080".parse().unwrap();

    let socket = TcpSocket::new_v4().unwrap();
    let mut stream = socket.connect(addr).await.unwrap();

    println!("connected");

    let (skt_read_half, mut skt_write_half) = stream.split();
    let mut skt_reader = BufReader::new(skt_read_half);

    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

    loop {
        let mut line1 = String::new();
        let mut line2 = String::new();

        println!("enter request...");

        tokio::select! {
            result = stdin_reader.read_line(&mut line1) => {
                result.unwrap();
                skt_write_half.write_all(line1.as_bytes()).await.unwrap();
            }
            result = skt_reader.read_line(&mut line2) => {
                result.unwrap();
                println!("{line2}");
            }
        }
    }
    
}
