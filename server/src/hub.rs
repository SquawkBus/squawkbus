use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use common::messages::{ForwardedSubscriptionRequest, Message, MulticastData, SubscriptionRequest};
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

        match msg {
            Message::AuthorizationRequest(_) => todo!(),
            Message::AuthorizationResponse(_) => todo!(),
            Message::ForwardedMulticastData(_) => todo!(),
            Message::ForwardedSubscriptionRequest(msg) => self.handle_forwarded_subscription_request(msg).await,
            Message::ForwardedUnicastData(_) => todo!(),
            Message::MulticastData(msg) => self.handle_multicast_data(msg).await,
            Message::NotificationRequest(_) => todo!(),
            Message::SubscriptionRequest(msg) => self.handle_subscription_request(msg).await,
            Message::UnicastData(_) => todo!(),
        }


        // let event = Arc::new(ServerEvent::OnMessage(msg));
        // for (client_addr, tx) in &self.clients {
        //     if *client_addr != addr {
        //         tx.send(event.clone()).await.unwrap();
        //     }
        // };
    }

    async fn handle_subscription_request(&mut self, msg: SubscriptionRequest) {

    }

    async fn handle_multicast_data(&mut self, msg: MulticastData) {

    }

    async fn handle_forwarded_subscription_request(&mut self, msg: ForwardedSubscriptionRequest) {

    }
}
