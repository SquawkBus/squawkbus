use std::{
    collections::HashSet,
    io::{self, Cursor},
};

use common::{
    io::Serializable,
    messages::{DataPacket, Message},
};

use crate::{
    authorization::AuthorizationManager,
    clients::ClientManager,
    constants::{SQUAWKBUS_CONTENT_TYPE, SUBSCRIPTION_TOPIC},
    publishing::PublisherManager,
    topic_tree::TopicTree,
};

pub struct SubscriptionManager {
    subscriptions: TopicTree,
    system_entitlements: HashSet<i32>,
}

impl SubscriptionManager {
    pub fn new() -> SubscriptionManager {
        SubscriptionManager {
            subscriptions: TopicTree::new(),
            system_entitlements: HashSet::new(),
        }
    }

    pub fn subscribers_for_topic(&self, topic: &str) -> Vec<&str> {
        self.subscriptions.subscribers(topic)
    }

    pub async fn handle_subscription_request(
        &mut self,
        id: &str,
        pattern: &str,
        is_add: bool,
        client_manager: &ClientManager,
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        if is_add {
            self.add_subscription(
                id,
                pattern,
                client_manager,
                publisher_manager,
                entitlements_manager,
            )
            .await
        } else {
            self.remove_subscription(
                id,
                pattern,
                client_manager,
                publisher_manager,
                entitlements_manager,
                false,
            )
            .await
        }
    }

    async fn add_subscription(
        &mut self,
        subscriber_id: &str,
        pattern: &str,
        client_manager: &ClientManager,
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        let count = self
            .subscriptions
            .add(pattern, subscriber_id)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if count > 1 {
            log::debug!("add_subscription: incrementing count for {pattern}");
        } else {
            log::debug!("add_subscription: creating new {pattern}");
        }

        self.notify_subscription(
            subscriber_id,
            pattern,
            count,
            client_manager,
            publisher_manager,
            entitlements_manager,
        )
        .await
    }

    async fn remove_subscription(
        &mut self,
        subscriber_id: &str,
        pattern: &str,
        client_manager: &ClientManager,
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
        is_subscriber_closed: bool,
    ) -> io::Result<()> {
        let Some(count) = self
            .subscriptions
            .remove(pattern, subscriber_id, is_subscriber_closed)
        else {
            return Ok(());
        };

        if count > 0 {
            log::debug!("removed one subscription for {subscriber_id} on {pattern}");
        } else {
            log::debug!("removed all subscriptions for {subscriber_id} on {pattern}");
        }

        self.notify_subscription(
            subscriber_id,
            pattern,
            count,
            client_manager,
            publisher_manager,
            entitlements_manager,
        )
        .await
    }

    pub async fn handle_close(
        &mut self,
        closed_client_id: &str,
        client_manager: &ClientManager,
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        let closed_client_topic_subscriptions = self.find_client_topics(closed_client_id);
        for topic in closed_client_topic_subscriptions {
            self.remove_subscription(
                closed_client_id,
                &topic,
                client_manager,
                publisher_manager,
                entitlements_manager,
                true,
            )
            .await?;
        }

        Ok(())
    }

    async fn notify_subscription(
        &mut self,
        subscriber_id: &str,
        pattern: &str,
        count: u32,
        client_manager: &ClientManager,
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        if pattern == SUBSCRIPTION_TOPIC {
            log::debug!("Client {subscriber_id} subscribed to {SUBSCRIPTION_TOPIC}");
            return Ok(());
        }

        let subscriber = client_manager.get(&subscriber_id).ok_or(io::Error::new(
            io::ErrorKind::Other,
            format!("unknown client {subscriber_id}"),
        ))?;

        let forwarded_subscription_request = Message::ForwardedSubscriptionRequest {
            host: subscriber.host.clone(),
            user: subscriber.user.clone(),
            client_id: subscriber_id.into(),
            topic: pattern.to_string(),
            count,
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        forwarded_subscription_request
            .serialize(&mut cursor)
            .expect("should serialize");

        let data = cursor.into_inner();

        let data_packet = DataPacket {
            name: "forwarded_subscription_request".to_string(),
            entitlement: 0,
            content_type: SQUAWKBUS_CONTENT_TYPE.to_string(),
            data,
        };

        publisher_manager
            .send_multicast_data_from(
                "squawkbus",
                "localhost",
                &self.system_entitlements,
                SUBSCRIPTION_TOPIC,
                vec![data_packet],
                self,
                client_manager,
                entitlements_manager,
            )
            .await?;

        Ok(())
    }

    fn find_client_topics(&self, client_id: &str) -> HashSet<String> {
        self.subscriptions.topics(client_id)
    }
}
