use std::{
    collections::{HashMap, HashSet},
    io,
    sync::Arc,
};

use common::messages::{DataPacket, ForwardedMulticastData, ForwardedUnicastData, Message};
use uuid::Uuid;

use crate::{clients::ClientManager, events::ServerEvent, subscriptions::SubscriptionManager};

pub struct PublisherManager {
    topics_by_publisher: HashMap<Uuid, HashSet<String>>,
    publishers_by_topic: HashMap<String, HashSet<Uuid>>,
}

impl PublisherManager {
    pub fn new() -> PublisherManager {
        PublisherManager {
            topics_by_publisher: HashMap::new(),
            publishers_by_topic: HashMap::new(),
        }
    }

    pub async fn handle_unicast_data(
        &mut self,
        publisher_id: Uuid,
        client_id: Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
        client_manager: &ClientManager,
    ) -> io::Result<()> {
        let Some(publisher) = client_manager.get(&publisher_id) else {
            println!("handle_unicast_data: no publisher {publisher_id}");
            return Ok(());
        };

        let Some(client) = client_manager.get(&client_id) else {
            println!("handle_unicast_data: no client {client_id}");
            return Ok(());
        };

        self.add_as_topic_publisher(&publisher_id, topic.as_str());

        let message = ForwardedUnicastData {
            client_id: publisher_id,
            host: publisher.host.clone(),
            user: publisher.user.clone(),
            topic,
            content_type,
            data_packets,
        };

        println!("handle_unicast_data: sending to client {client_id} message {message:?}");

        let event = Arc::new(ServerEvent::OnMessage(Message::ForwardedUnicastData(
            message,
        )));

        client
            .tx
            .send(event.clone())
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        println!("handle_unicast_data: ...sent");

        Ok(())
    }

    pub async fn handle_multicast_data(
        &mut self,
        publisher_id: &Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
        subscription_manager: &SubscriptionManager,
        client_manager: &ClientManager,
    ) -> io::Result<()> {
        let Some(subscribers) = subscription_manager.subscribers_for_topic(topic.as_str()) else {
            println!("handle_multicast_data: no topic {topic}");
            return Ok(());
        };

        let Some(publisher) = client_manager.get(publisher_id) else {
            println!("handle_multicast_data: not publisher {publisher_id}");
            return Ok(());
        };

        self.add_as_topic_publisher(publisher_id, topic.as_str());

        let message = ForwardedMulticastData {
            host: publisher.host.clone(),
            user: publisher.user.clone(),
            topic,
            content_type,
            data_packets,
        };

        println!("handle_multicast_data: sending message {message:?} to clients ...");

        let event = Arc::new(ServerEvent::OnMessage(Message::ForwardedMulticastData(
            message,
        )));

        for subscriber_id in subscribers.keys() {
            if let Some(subscriber) = client_manager.get(subscriber_id) {
                println!("handle_multicast_data: ... {subscriber_id}");
                subscriber
                    .tx
                    .send(event.clone())
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
        }

        println!("handle_multicast_data: ...sent");

        Ok(())
    }

    fn add_as_topic_publisher(&mut self, publisher_id: &Uuid, topic: &str) {
        let topics = self
            .topics_by_publisher
            .entry(publisher_id.clone())
            .or_default();
        if !topics.contains(topic) {
            topics.insert(topic.to_string());
        }

        let publishers = self
            .publishers_by_topic
            .entry(topic.to_string())
            .or_default();
        if !publishers.contains(publisher_id) {
            publishers.insert(publisher_id.clone());
        }
    }

    pub async fn handle_close(
        &mut self,
        closed_client_id: &Uuid,
        client_manager: &ClientManager,
        subscription_manager: &SubscriptionManager,
    ) -> io::Result<()> {
        let topics_without_publishers = remove_publisher(
            closed_client_id,
            &mut self.topics_by_publisher,
            &mut self.publishers_by_topic,
        );

        if topics_without_publishers.len() > 0 {
            notify_subscribers_of_stale_topics(
                closed_client_id,
                topics_without_publishers,
                client_manager,
                subscription_manager,
            )
            .await
        } else {
            Ok(())
        }
    }
}

fn remove_publisher(
    closed_client_id: &Uuid,
    topics_by_publisher: &mut HashMap<Uuid, HashSet<String>>,
    publishers_by_topic: &mut HashMap<String, HashSet<Uuid>>,
) -> Vec<String> {
    let mut topics_without_publishers: Vec<String> = Vec::new();

    // Find all the topics for which this client has published.
    if let Some(publisher_topics) = topics_by_publisher.remove(closed_client_id) {
        for topic in publisher_topics {
            if let Some(topic_publishers) = publishers_by_topic.get_mut(topic.as_str()) {
                topic_publishers.remove(closed_client_id);
                if topic_publishers.len() == 0 {
                    topics_without_publishers.push(topic);
                }
            }
        }
    }

    for topic in &topics_without_publishers {
        publishers_by_topic.remove(topic.as_str());
    }

    topics_without_publishers
}

async fn notify_subscribers_of_stale_topics(
    closed_client_id: &Uuid,
    topics_without_publishers: Vec<String>,
    client_manager: &ClientManager,
    subscription_manager: &SubscriptionManager,
) -> io::Result<()> {
    let Some(publisher) = client_manager.get(closed_client_id) else {
        println!("handle_close: not publisher {closed_client_id}");
        return Ok(());
    };

    for topic in topics_without_publishers {
        let stale_data_message = ForwardedMulticastData {
            host: publisher.host.clone(),
            user: publisher.user.clone(),
            topic: topic.clone(),
            content_type: String::from("application/octet-stream"),
            data_packets: Vec::new(),
        };

        let event = Arc::new(ServerEvent::OnMessage(Message::ForwardedMulticastData(
            stale_data_message,
        )));

        if let Some(subscribers) = subscription_manager.subscribers_for_topic(topic.as_str()) {
            for subscriber_id in subscribers.keys() {
                if let Some(subscriber) = client_manager.get(subscriber_id) {
                    println!("handle_close: sending stale to {subscriber_id}");
                    subscriber
                        .tx
                        .send(event.clone())
                        .await
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                }
            }
        }
    }

    Ok(())
}
