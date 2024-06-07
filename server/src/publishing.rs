use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use common::messages::{DataPacket, ForwardedMulticastData, ForwardedUnicastData, Message};
use uuid::Uuid;

use crate::{clients::ClientManager, events::ServerEvent, subscriptions::SubscriptionManager};

pub struct PublisherManager {
    publisher_topics: HashMap<Uuid, HashSet<String>>,
}

impl PublisherManager {
    pub fn new() -> PublisherManager {
        PublisherManager {
            publisher_topics: HashMap::new(),
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
    ) {
        let Some(publisher) = client_manager.get(&publisher_id) else {
            println!("handle_unicast_data: no publisher {publisher_id}");
            return;
        };

        let Some(client) = client_manager.get(&client_id) else {
            println!("handle_unicast_data: no client {client_id}");
            return;
        };

        self.add_topic(&publisher_id, topic.as_str());

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

        client.tx.send(event.clone()).await.unwrap();

        println!("handle_unicast_data: ...sent");
    }

    pub async fn handle_multicast_data(
        &mut self,
        publisher_id: &Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
        subscription_manager: &SubscriptionManager,
        client_manager: &ClientManager,
    ) {
        let Some(subscribers) = subscription_manager.subscribers_for_topic(topic.as_str()) else {
            println!("handle_multicast_data: no topic {topic}");
            return;
        };

        let Some(publisher) = client_manager.get(publisher_id) else {
            println!("handle_multicast_data: not publisher {publisher_id}");
            return;
        };

        self.add_topic(publisher_id, topic.as_str());

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
                subscriber.tx.send(event.clone()).await.unwrap();
            }
        }

        println!("handle_multicast_data: ...sent");
    }

    fn add_topic(&mut self, publisher_id: &Uuid, topic: &str) {
        let topics = self
            .publisher_topics
            .entry(publisher_id.clone())
            .or_default();
        if !topics.contains(topic) {
            topics.insert(topic.to_string());
        }
    }
}
