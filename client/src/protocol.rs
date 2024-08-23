use std::collections::HashSet;

use tokio::io::{split, AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use common::messages::{
    DataPacket, Message, MulticastData, NotificationRequest, SubscriptionRequest,
};

use crate::authentication::authenticate;

pub async fn communicate<S>(
    stream: S,
    mode: &String,
    username: &Option<String>,
    password: &Option<String>,
) where
    S: AsyncRead + AsyncWrite,
{
    println!("connected");

    let (skt_read_half, mut skt_write_half) = split(stream);
    let mut skt_reader = BufReader::new(skt_read_half);

    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

    authenticate(&mut skt_write_half, mode, username, password)
        .await
        .unwrap();

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
    let args: Vec<&str> = line.trim().split(' ').collect();
    match args[0] {
        "publish" => handle_publish(args),
        "subscribe" => handle_subscribe(args),
        "notify" => handle_notify(args),
        _ => Err("usage: publish/subscribe/notify"),
    }
}

fn handle_publish(args: Vec<&str>) -> Result<Message, &'static str> {
    if args.len() < 4 || args.len() % 2 == 1 {
        return Err("usage: publish <topic> ((<entitlements> | '_') <message>)+");
    }

    let topic = args[1];
    let mut i = 2;
    let mut data_packets: Vec<DataPacket> = Vec::new();
    while i < args.len() {
        let entitlements: HashSet<i32> = match args[i] {
            "_" => HashSet::new(),
            values => values
                .split(',')
                .map(|x| x.parse().expect("should be an integer"))
                .collect(),
        };
        i += 1;

        let message = args[i];
        data_packets.push(DataPacket::new(entitlements, Vec::from(message.as_bytes())));
        i += 1;
    }
    let message = MulticastData {
        topic: topic.to_string(),
        content_type: String::from("text/plain"),
        data_packets,
    };
    Ok(Message::MulticastData(message))
}

fn handle_subscribe(args: Vec<&str>) -> Result<Message, &'static str> {
    if args.len() != 2 {
        return Err("usage: subscribe <topic>");
    }
    let topic = args[1].to_string();
    let message = SubscriptionRequest {
        topic,
        is_add: true,
    };
    Ok(Message::SubscriptionRequest(message))
}

fn handle_notify(args: Vec<&str>) -> Result<Message, &'static str> {
    if args.len() != 2 {
        return Err("usage: subscribe <topic>");
    }
    let pattern = args[1].to_string();
    let message = NotificationRequest {
        pattern: pattern.to_string(),
        is_add: true,
    };
    Ok(Message::NotificationRequest(message))
}
