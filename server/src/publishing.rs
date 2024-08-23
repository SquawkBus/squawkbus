use std::{
    collections::{HashMap, HashSet},
    io,
};

use common::messages::{DataPacket, ForwardedMulticastData, ForwardedUnicastData, Message};
use uuid::Uuid;

use crate::{
    authorization::{AuthorizationManager, Role},
    clients::ClientManager,
    events::ServerEvent,
    subscriptions::SubscriptionManager,
};

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

    fn get_authorized_data(
        &self,
        data_packets: Vec<DataPacket>,
        entitlements: &HashSet<i32>,
    ) -> Vec<DataPacket> {
        let mut authorised_data_packets = Vec::new();
        for data_packet in data_packets {
            if data_packet.is_authorized(&entitlements) {
                authorised_data_packets.push(data_packet)
            }
        }
        authorised_data_packets
    }

    /// Send data from one client to another.
    pub async fn send_unicast_data(
        &mut self,
        sender_id: Uuid,
        receiver_id: Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
        allow_empty_message: bool,
        client_manager: &ClientManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        let Some(sender) = client_manager.get(&sender_id) else {
            log::debug!("send_unicast_data: no sender client {sender_id} - skipping");
            return Ok(());
        };

        let Some(receiver) = client_manager.get(&receiver_id) else {
            log::debug!("send_unicast_data: no receiver client {receiver_id} - skipping");
            return Ok(());
        };

        // Get the entitlements.
        let sender_entitlements = entitlements_manager.entitlements(
            sender.user.as_str(),
            topic.as_str(),
            Role::Publisher,
        );
        let receiver_entitlements = entitlements_manager.entitlements(
            receiver.user.as_str(),
            topic.as_str(),
            Role::Subscriber,
        );
        let entitlements: HashSet<i32> = sender_entitlements
            .intersection(&receiver_entitlements)
            .cloned()
            .collect();

        if !sender_entitlements.is_empty() && entitlements.is_empty() {
            // Entitlements only operate if the sender has entitlements.
            log::debug!(
                "send_unicast_data: no entitlements from {} to {} for {}",
                sender.user,
                receiver.user,
                topic
            );
            return Ok(());
        }

        let auth_data_packets = self.get_authorized_data(data_packets, &entitlements);

        if !allow_empty_message && auth_data_packets.is_empty() {
            log::debug!(
                "send_unicast_data: empty message from {} to {} for {} - skipping",
                sender.user,
                receiver.user,
                topic
            );
            return Ok(());
        }

        self.add_as_topic_publisher(&sender_id, topic.as_str());

        let message = ForwardedUnicastData {
            client_id: sender_id,
            host: sender.host.clone(),
            user: sender.user.clone(),
            topic,
            content_type,
            data_packets: auth_data_packets,
        };

        log::debug!("send_unicast_data: sending to client {receiver_id} message {message:?}");

        let event = ServerEvent::OnMessage(Message::ForwardedUnicastData(message));

        receiver
            .tx
            .send(event)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        log::debug!("send_unicast_data: ...sent");

        Ok(())
    }

    /// Send data to clients that subscribe to a topic.
    pub async fn send_multicast_data(
        &mut self,
        publisher_id: &Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
        allow_empty_message: bool,
        subscription_manager: &SubscriptionManager,
        client_manager: &ClientManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        let subscribers = subscription_manager.subscribers_for_topic(topic.as_str());
        if subscribers.is_empty() {
            log::debug!("send_multicast_data: no topic {topic}");
            return Ok(());
        }

        let Some(publisher) = client_manager.get(publisher_id) else {
            log::debug!("send_multicast_data: not publisher {publisher_id}");
            return Ok(());
        };

        let publisher_entitlements = entitlements_manager.entitlements(
            publisher.user.as_str(),
            topic.as_str(),
            Role::Publisher,
        );

        self.add_as_topic_publisher(publisher_id, topic.as_str());

        for subscriber_id in &subscribers {
            if let Some(subscriber) = client_manager.get(subscriber_id) {
                log::debug!("send_multicast_data: ... {subscriber_id}");

                let subscriber_entitlements = entitlements_manager.entitlements(
                    subscriber.user.as_str(),
                    topic.as_str(),
                    Role::Subscriber,
                );
                let entitlements: HashSet<i32> = publisher_entitlements
                    .intersection(&subscriber_entitlements)
                    .cloned()
                    .collect();

                if !publisher_entitlements.is_empty() && entitlements.is_empty() {
                    // Entitlements only operate if the publisher has entitlements.
                    log::debug!(
                        "send_multicast_data: no entitlements from {} to {} for {}",
                        publisher.user,
                        subscriber.user,
                        topic
                    );
                    continue;
                }

                let auth_data_packets =
                    self.get_authorized_data(data_packets.clone(), &entitlements);

                if !allow_empty_message && auth_data_packets.is_empty() {
                    log::debug!(
                        "send_multicast_data: empty message from {} to {} for {}",
                        publisher.user,
                        subscriber.user,
                        topic
                    );
                    continue;
                }

                let message = ForwardedMulticastData {
                    host: publisher.host.clone(),
                    user: publisher.user.clone(),
                    topic: topic.clone(),
                    content_type: content_type.clone(),
                    data_packets: auth_data_packets,
                };

                log::debug!(
                    "send_multicast_data: sending message {message:?} to client {subscriber_id}"
                );

                let event = ServerEvent::OnMessage(Message::ForwardedMulticastData(message));

                subscriber
                    .tx
                    .send(event)
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
        }

        log::debug!("send_multicast_data: ...sent");

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
        log::debug!("handle_close: not publisher {closed_client_id}");
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

        let subscribers = subscription_manager.subscribers_for_topic(topic.as_str());
        for subscriber_id in &subscribers {
            if let Some(subscriber) = client_manager.get(subscriber_id) {
                log::debug!("handle_close: sending stale to {subscriber_id}");

                let event = ServerEvent::OnMessage(Message::ForwardedMulticastData(
                    stale_data_message.clone(),
                ));

                subscriber
                    .tx
                    .send(event)
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
        }
    }

    Ok(())
}
