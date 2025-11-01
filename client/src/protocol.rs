use std::collections::{HashMap, HashSet};

use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use common::{
    messages::{DataPacket, Message},
    MessageSocket, MessageStream,
};

use crate::authentication::authenticate;

pub async fn communicate<S>(
    stream: S,
    mode: &String,
    username: &Option<String>,
    password: &Option<String>,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    println!("connected");

    let mut stream = MessageSocket::new(stream);

    let stdin = tokio::io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

    let client_id = authenticate(&mut stream, mode, username, password)
        .await
        .unwrap();
    println!("Authenticted as {client_id}");

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
                        stream.write(&message).await.unwrap();
                    },
                    Err(message) => {
                        println!("{message}");
                    }
                }
            }
            // response
            result = stream.read() => {
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
        data_packets.push(DataPacket::new(
            entitlements,
            HashMap::from([("content-type".to_string(), "text/plain".to_string())]),
            Vec::from(message.as_bytes()),
        ));
        i += 1;
    }
    let message = Message::MulticastData {
        topic: topic.to_string(),
        data_packets,
    };
    Ok(message)
}

fn handle_subscribe(args: Vec<&str>) -> Result<Message, &'static str> {
    if args.len() != 2 {
        return Err("usage: subscribe <topic>");
    }
    let topic = args[1].to_string();
    let message = Message::SubscriptionRequest {
        topic,
        is_add: true,
    };
    Ok(message)
}
