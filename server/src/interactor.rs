use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};

use crate::events::{ClientEvent, ServerEvent};

pub async fn interactor_run(mut socket: TcpStream, addr: SocketAddr, client_tx: Sender<ClientEvent>) {
    let (server_tx, mut client_rx) = mpsc::channel::<ServerEvent>(32);

    client_tx.send(ClientEvent::OnConnect(addr.clone(), server_tx)).await.unwrap();

    let (read_half, mut write_half) = socket.split();

    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break;
                }

                client_tx.send(ClientEvent::OnMessage(addr, line.clone())).await.unwrap();
                line.clear();
            }
            result = client_rx.recv() => {
                match result {
                    Some(event) => {
                        match event {
                            ServerEvent::OnMessage(line) => {
                                write_half.write_all(line.as_bytes()).await.unwrap();
                            }
                        }
                    },
                    None => todo!(),
                }
            }
        }
    }
}