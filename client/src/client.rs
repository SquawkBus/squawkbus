use std::future::Future;
use std::io;
use std::net::ToSocketAddrs;
use std::path::PathBuf;

use common::messages::DataPacket;
use common::messages::Message;
use common::messages::MulticastData;
use common::messages::NotificationRequest;
use common::messages::SubscriptionRequest;
use common::messages::UnicastData;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::{self, Sender};
use uuid::Uuid;

pub trait ClientCallbacks {
    fn on_data(
        &mut self,
        publisher: String,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) -> impl Future<Output = ()>;
    fn on_forwarded_subscription(
        &mut self,
        user: String,
        topic: String,
        is_add: bool,
    ) -> impl Future<Output = ()>;
}

pub struct Client<C>
where
    C: ClientCallbacks,
{
    callbacks: Box<C>,
    sender: Sender<Message>,
}

trait ClientProtocol {
    fn send(
        &mut self,
        client_id: Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) -> impl Future<Output = io::Result<()>>;
    fn publish(
        &mut self,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) -> impl Future<Output = io::Result<()>>;
    fn add_subscription(&mut self, topic: String) -> impl Future<Output = io::Result<()>>;
    fn remove_subscription(&mut self, topic: String) -> impl Future<Output = io::Result<()>>;
    fn remove_notification(&mut self, topic: String) -> impl Future<Output = io::Result<()>>;
    fn add_notification(&mut self, topic: String) -> impl Future<Output = io::Result<()>>;
}

struct Communicator<S>
where
    S: AsyncRead + AsyncWrite,
{
    stream: S,
    sender: Sender<Message>,
}

impl<S> Communicator<S>
where
    S: AsyncRead + AsyncWrite,
{
    async fn send_message(&mut self, message: Message) -> io::Result<()> {
        self.sender
            .send(message)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }

    async fn send_unicast_request(
        &mut self,
        client_id: Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) -> io::Result<()> {
        let message = Message::UnicastData(UnicastData {
            client_id,
            topic,
            content_type,
            data_packets,
        });
        self.send_message(message).await
    }

    async fn send_multicast_request(
        &mut self,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) -> io::Result<()> {
        let message = Message::MulticastData(MulticastData {
            topic,
            content_type,
            data_packets,
        });
        self.send_message(message).await
    }

    async fn send_subscription_request(&mut self, topic: String, is_add: bool) -> io::Result<()> {
        let message = Message::SubscriptionRequest(SubscriptionRequest { topic, is_add });
        self.send_message(message).await
    }

    async fn send_notification_request(&mut self, pattern: String, is_add: bool) -> io::Result<()> {
        let message = Message::NotificationRequest(NotificationRequest { pattern, is_add });
        self.send_message(message).await
    }
}

impl<S> ClientProtocol for Communicator<S>
where
    S: AsyncRead + AsyncWrite,
{
    fn send(
        &mut self,
        client_id: Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) -> impl Future<Output = io::Result<()>> {
        async move {
            self.send_unicast_request(client_id, topic, content_type, data_packets)
                .await
        }
    }

    fn publish(
        &mut self,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) -> impl Future<Output = io::Result<()>> {
        async move {
            self.send_multicast_request(topic, content_type, data_packets)
                .await
        }
    }

    fn add_subscription(&mut self, topic: String) -> impl Future<Output = io::Result<()>> {
        async move { self.send_subscription_request(topic, true).await }
    }

    fn remove_subscription(&mut self, topic: String) -> impl Future<Output = io::Result<()>> {
        async move { self.send_subscription_request(topic, false).await }
    }

    fn add_notification(&mut self, pattern: String) -> impl Future<Output = io::Result<()>> {
        async move { self.send_notification_request(pattern, true).await }
    }

    fn remove_notification(&mut self, pattern: String) -> impl Future<Output = io::Result<()>> {
        async move { self.send_notification_request(pattern, false).await }
    }
}

impl<T: ClientCallbacks> Client<T> {
    pub fn new(
        host: &str,
        port: u16,
        tls: bool,
        cafile: &Option<PathBuf>,
        callbacks: Box<T>,
    ) -> io::Result<Self> {
        let endpoint = format!("{}:{}", host, port);

        let addr = endpoint
            .to_socket_addrs()?
            .next()
            .ok_or(format!("failed to resolve {}", host))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let (sender, receiver) = mpsc::channel::<Message>(10);

        Ok(Client { callbacks, sender })
    }
}
