use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use common::messages::Message;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::events::{ClientEvent, ServerEvent};

pub struct Hub {
    clients: HashMap<SocketAddr, Sender<Arc<ServerEvent>>>
}

impl Hub {
    pub fn new() -> Hub {
        Hub {
            clients: HashMap::new()
        }
    }

    pub async fn run(&mut self, mut server_rx: Receiver<ClientEvent>) {
        loop {
            let msg = server_rx.recv().await.unwrap();
            match msg {
                ClientEvent::OnConnect(addr, server_tx) => self.handle_connect(addr, server_tx),
                ClientEvent::OnMessage(addr, msg) => self.handle_message(addr, msg).await,
            }
        }    
    }

    fn handle_connect(&mut self, addr: SocketAddr, server_tx: Sender<Arc<ServerEvent>>) {
        println!("client connected from {addr}");
        self.clients.insert(addr, server_tx);
    }

    async fn handle_message(&mut self, addr: SocketAddr, msg: Message) {
        println!("Received message from {addr}: \"{msg:?}\"");
        let event = Arc::new(ServerEvent::OnMessage(msg));
        for (client_addr, tx) in &self.clients {
            if *client_addr != addr {
                tx.send(event.clone()).await.unwrap();
            }
        };
    }
}
