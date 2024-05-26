use std::{collections::HashMap, net::SocketAddr};

use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}, sync::mpsc::{self, Receiver, Sender}};

enum ClientEvent {
    OnConnect(SocketAddr, Sender<ServerEvent>),
    OnMessage(SocketAddr, String)
}

enum ServerEvent {
    OnMessage(String),
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);

    // Create a hub that listens to clients
    tokio::spawn(async move {
            hub_run(server_rx).await;
        }
    );

    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let client_tx = client_tx.clone();

        tokio::spawn(async move {
            interactor_run(socket, addr, client_tx).await;
        });
    }
}

async fn hub_run(mut server_rx: Receiver<ClientEvent>) {
    let mut clients: HashMap<SocketAddr, Sender<ServerEvent>> = HashMap::new();

    loop {
        let msg = server_rx.recv().await.unwrap();
        match msg {
            ClientEvent::OnConnect(addr, server_tx) => {
                println!("client connected from {addr}");
                clients.insert(addr, server_tx);
            },
            ClientEvent::OnMessage(addr, line) => {
                println!("Received message from {addr}: \"{line}\"");
                for (client_addr, tx) in &clients {
                    if *client_addr != addr {
                        tx.send(ServerEvent::OnMessage(line.clone())).await.unwrap();
                    }
                };
            },
        }
    }
}

async fn interactor_run(mut socket: TcpStream, addr: SocketAddr, client_tx: Sender<ClientEvent>) {
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