use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};

use uuid::Uuid;

use common::messages::Message;

use crate::events::{ClientEvent, ServerEvent};

pub struct Interactor {
    pub id: Uuid
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor {
            id: Uuid::new_v4()
        }
    }

    pub async fn run(&self, mut socket: TcpStream, addr: SocketAddr, hub: Sender<ClientEvent>) {
        let (tx, mut rx) = mpsc::channel::<Arc<ServerEvent>>(32);
    
        hub.send(ClientEvent::OnConnect(addr.clone(), tx)).await.unwrap();
    
        let (read_half, mut write_half) = socket.split();
    
        let mut reader = BufReader::new(read_half);
    
        loop {
            tokio::select! {
                // forward client to hub
                result = Message::read(&mut reader) => {
                    let msg = result.unwrap();
                    hub.send(ClientEvent::OnMessage(addr, msg)).await.unwrap();
                }
                // forward hub to client
                result = rx.recv() => {
                    match result {
                        Some(event) => {
                            match event.as_ref() {
                                ServerEvent::OnMessage(message) => {
                                    message.write(&mut write_half).await.unwrap();
                                    // write_half.write_all(line.as_bytes()).await.unwrap();
                                }
                            }
                        },
                        None => todo!(),
                    }
                }
            }
        }
    }}
