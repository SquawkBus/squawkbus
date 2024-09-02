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
use tokio::io::{split, AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;

use crate::authentication::authenticate;
use crate::tls::create_tls_stream;

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

pub struct Client<S, C>
where
    S: AsyncRead + AsyncWrite,
    C: ClientCallbacks,
{
    callbacks: Box<C>,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    reader: ReadHalf<S>,
    writer: WriteHalf<S>,
}

impl<S, C> Client<S, C>
where
    S: AsyncRead + AsyncWrite,
    C: ClientCallbacks,
{
    pub async fn start(
        stream: S,
        callbacks: Box<C>,
        mode: &String,
        username: &Option<String>,
        password: &Option<String>,
    ) -> io::Result<Self> {
        let (reader, mut writer) = split(stream);
        //let mut skt_reader = BufReader::new(skt_read_half);
        let (tx, rx) = mpsc::channel::<Message>(32);

        authenticate(&mut writer, mode, username, password).await?;

        Ok(Client {
            callbacks,
            tx,
            rx,
            reader,
            writer,
        })
    }

    async fn send_message(&mut self, message: Message) -> io::Result<()> {
        self.tx
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

impl<S, C> ClientProtocol for Client<S, C>
where
    S: AsyncRead + AsyncWrite,
    C: ClientCallbacks,
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

pub async fn connect<S, C>(
    host: &str,
    port: u16,
    tls: bool,
    cafile: &Option<PathBuf>,
    authentication_mode: &String,
    username: &Option<String>,
    password: &Option<String>,
    callbacks: Box<C>,
) -> io::Result<Box<Client<S, C>>>
where
    C: ClientCallbacks,
    S: AsyncRead + AsyncWrite,
{
    let endpoint = format!("{}:{}", host, port);

    let addr = endpoint
        .to_socket_addrs()?
        .next()
        .ok_or(format!("failed to resolve {}", host))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let stream = TcpStream::connect(&addr).await?;

    let client = match tls {
        true => {
            let stream = create_tls_stream(host, cafile, stream).await?;
            let client = Box::new(
                Client::start(stream, callbacks, authentication_mode, username, password).await?,
            );
            client
        }
        false => {
            let client = Box::new(
                Client::start(stream, callbacks, authentication_mode, username, password).await?,
            );
            client
        }
    };

    Ok(client)
}
