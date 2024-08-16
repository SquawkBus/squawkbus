use std::error::Error;
use std::fs::File;
use std::io;
use std::sync::Arc;
use std::{collections::HashSet, net::ToSocketAddrs};

use tokio::io::AsyncWriteExt;
use tokio_rustls::{rustls, TlsConnector};

use options::Options;
use tokio::{
    io::{split, AsyncBufReadExt, BufReader},
    net::TcpStream,
};

use common::messages::{
    DataPacket, Message, MulticastData, NotificationRequest, SubscriptionRequest,
};

mod options;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("client");

    let options = Options::load();

    let endpoint = format!("{}:{}", options.host.as_str(), options.port);

    let addr = endpoint
        .to_socket_addrs()?
        .next()
        .ok_or(format!("failed to resolve {}", options.host.as_str()))?;

    let mut root_cert_store = rustls::RootCertStore::empty();
    if let Some(cafile) = &options.cafile {
        let mut pem = io::BufReader::new(File::open(cafile).expect("Should open cert file"));
        for cert in rustls_pemfile::certs(&mut pem) {
            root_cert_store.add(cert?).unwrap();
        }
    } else {
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth(); // i guess this was previously the default?
    let connector = TlsConnector::from(Arc::new(config));

    let socket = TcpStream::connect(&addr).await?;

    let domain = pki_types::ServerName::try_from(options.host.as_str())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dns name"))?
        .to_owned();

    let stream = connector.connect(domain, socket).await?;

    println!("connected");

    let (skt_read_half, mut skt_write_half) = split(stream);
    let mut skt_reader = BufReader::new(skt_read_half);

    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

    // Handshake
    skt_write_half.write_all("nobody\n".as_bytes()).await?;
    skt_write_half.write_all("trustno1\n".as_bytes()).await?;

    loop {
        let mut request_line = String::new();

        println!("Enter request:");
        println!("\tpublish <topic> <entitlements> <message>");
        println!("\tsubscribe <topic>");
        println!("\tnotify <pattern>");

        tokio::select! {
            // request
            result = stdin_reader.read_line(&mut request_line) => {
                result.unwrap();
                match parse_message(request_line.as_str()) {
                    Ok(message) => {
                        message.write(&mut skt_write_half).await.unwrap();
                    },
                    Err(message) => {
                        println!("{message}");
                    }
                }
            }
            // response
            result = Message::read(&mut skt_reader) => {
                let message = result.unwrap();
                println!("Received message {message:?}");
            }
        }
    }
}

fn parse_message(line: &str) -> Result<Message, &'static str> {
    let parts: Vec<&str> = line.trim().split(' ').collect();
    match parts[0] {
        "publish" => {
            if parts.len() != 4 {
                Err("usage: publish <topic> <entitlements> <message>")
            } else {
                let topic = parts[1];
                let entitlements: HashSet<i32> = parts[2]
                    .split(',')
                    .map(|x| x.parse().expect("should be an integer"))
                    .collect();
                let message = parts[3];
                let message = create_multicast_message(topic, entitlements, message);
                Ok(Message::MulticastData(message))
            }
        }
        "subscribe" => {
            if parts.len() != 2 {
                Err("usage: subscribe <topic>")
            } else {
                let message = create_subscription_message(parts[1]);
                Ok(Message::SubscriptionRequest(message))
            }
        }
        "notify" => {
            if parts.len() != 2 {
                Err("usage: subscribe <topic>")
            } else {
                let message = create_notification_message(parts[1]);
                Ok(Message::NotificationRequest(message))
            }
        }
        _ => Err("usage: publish/subscribe/notify"),
    }
}

fn create_multicast_message(
    topic: &str,
    entitlements: HashSet<i32>,
    message: &str,
) -> MulticastData {
    MulticastData {
        topic: topic.to_string(),
        content_type: String::from("text/plain"),
        data_packets: vec![DataPacket::new(entitlements, Vec::from(message.as_bytes()))],
    }
}

fn create_subscription_message(topic: &str) -> SubscriptionRequest {
    SubscriptionRequest {
        topic: topic.to_string(),
        is_add: true,
    }
}

fn create_notification_message(pattern: &str) -> NotificationRequest {
    NotificationRequest {
        pattern: pattern.to_string(),
        is_add: true,
    }
}
