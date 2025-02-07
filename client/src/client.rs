use std::io;
use std::net::ToSocketAddrs;
use std::path::PathBuf;

use futures::future::BoxFuture;

use common::messages::DataPacket;
use common::messages::Message;
use common::messages::MulticastData;
use common::messages::NotificationRequest;
use common::messages::SubscriptionRequest;
use common::messages::UnicastData;
use tokio::io::{split, AsyncRead, AsyncWrite, BufReader, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_rustls::client;

use crate::authentication::authenticate;
use crate::tls::create_tls_stream;

pub trait ClientCallbacks {
    fn on_data(&mut self, topic: String, data_packets: Vec<DataPacket>) -> BoxFuture<'_, ()>;
    fn on_forwarded_subscription(
        &mut self,
        user: String,
        topic: String,
        is_add: bool,
    ) -> BoxFuture<'_, ()>;
}

pub trait ClientProtocol {
    fn send(
        &mut self,
        client_id: String,
        topic: String,
        data_packets: Vec<DataPacket>,
    ) -> BoxFuture<'_, io::Result<()>>;
    fn publish(
        &mut self,
        topic: String,
        data_packets: Vec<DataPacket>,
    ) -> BoxFuture<'_, io::Result<()>>;
    fn add_subscription(&mut self, topic: String) -> BoxFuture<'_, io::Result<()>>;
    fn remove_subscription(&mut self, topic: String) -> BoxFuture<'_, io::Result<()>>;
    fn remove_notification(&mut self, topic: String) -> BoxFuture<'_, io::Result<()>>;
    fn add_notification(&mut self, topic: String) -> BoxFuture<'_, io::Result<()>>;
}

pub struct Client<S>
where
    S: AsyncRead + AsyncWrite + Send,
{
    callbacks: Box<dyn ClientCallbacks + Send>,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    reader: BufReader<ReadHalf<S>>,
    writer: WriteHalf<S>,
}

impl<S> Client<S>
where
    S: AsyncRead + AsyncWrite + Send,
{
    pub async fn start(
        stream: S,
        callbacks: Box<dyn ClientCallbacks + Send>,
        mode: &String,
        username: &Option<String>,
        password: &Option<String>,
    ) -> io::Result<Self> {
        let (reader, mut writer) = split(stream);
        //let mut skt_reader = BufReader::new(skt_read_half);
        let (tx, rx) = mpsc::channel::<Message>(32);

        authenticate(&mut writer, mode, username, password).await?;

        let mut client = Client {
            callbacks,
            tx,
            rx,
            reader: BufReader::new(reader),
            writer,
        };

        Ok(client)
    }

    fn send_message(&mut self, message: Message) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(async move {
            self.tx
                .send(message)
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            Ok(())
        })
    }

    async fn send_unicast_request(
        &mut self,
        client_id: String,
        topic: String,
        data_packets: Vec<DataPacket>,
    ) -> io::Result<()> {
        let message = Message::UnicastData(UnicastData {
            client_id,
            topic,
            data_packets,
        });
        self.send_message(message).await
    }

    async fn send_multicast_request(
        &mut self,
        topic: String,
        data_packets: Vec<DataPacket>,
    ) -> io::Result<()> {
        let message = Message::MulticastData(MulticastData {
            topic,
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

    async fn handle_message(&mut self, message: Message) {
        match message {
            Message::UnicastData(msg) => self.callbacks.on_data(msg.topic, msg.data_packets).await,
            Message::MulticastData(msg) => {
                self.callbacks.on_data(msg.topic, msg.data_packets).await
            }
            Message::ForwardedSubscriptionRequest(msg) => {
                self.callbacks
                    .on_forwarded_subscription(msg.client_id, msg.topic, msg.is_add)
                    .await
            }
            _ => todo!(),
        };
    }

    async fn process(&mut self) {
        loop {
            tokio::select! {
                result = self.rx.recv() => {
                    // Send a message to the server.
                    let message = result.unwrap();
                    message.write(&mut self.writer).await.unwrap();
                }
                result = Message::read(&mut self.reader) => {
                    let message = result.unwrap();
                    self.handle_message(message).await;
                }
            }
        }
    }
}

impl<S> ClientProtocol for Client<S>
where
    S: AsyncRead + AsyncWrite + Send,
{
    fn send(
        &mut self,
        client_id: String,
        topic: String,
        data_packets: Vec<DataPacket>,
    ) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(async move {
            self.send_unicast_request(client_id, topic, data_packets)
                .await
        })
    }

    fn publish(
        &mut self,
        topic: String,
        data_packets: Vec<DataPacket>,
    ) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(async move { self.send_multicast_request(topic, data_packets).await })
    }

    fn add_subscription(&mut self, topic: String) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(async move { self.send_subscription_request(topic, true).await })
    }

    fn remove_subscription(&mut self, topic: String) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(async move { self.send_subscription_request(topic, false).await })
    }

    fn add_notification(&mut self, pattern: String) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(async move { self.send_notification_request(pattern, true).await })
    }

    fn remove_notification(&mut self, pattern: String) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(async move { self.send_notification_request(pattern, false).await })
    }
}

pub async fn connect<S>(
    host: &str,
    port: u16,
    tls: bool,
    cafile: &Option<PathBuf>,
    authentication_mode: &String,
    username: &Option<String>,
    password: &Option<String>,
    callbacks: Box<dyn ClientCallbacks + Send>,
) -> io::Result<Box<dyn ClientProtocol>>
where
    S: AsyncRead + AsyncWrite + Send,
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
            let client: Box<dyn ClientProtocol> = Box::from(
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
