use std::{collections::HashMap, io};

use regex::Regex;

use common::messages::{ForwardedSubscriptionRequest, Message, NotificationRequest};

use crate::{clients::ClientManager, events::ServerEvent, subscriptions::SubscriptionManager};

struct Notification {
    regex: Regex,
    listeners: HashMap<String, u32>,
}

impl Notification {
    pub fn new(topic: &str) -> io::Result<Self> {
        let regex = Regex::new(topic).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Notification {
            regex,
            listeners: HashMap::new(),
        })
    }
}

pub struct NotificationManager {
    notifications: HashMap<String, Notification>,
}

impl NotificationManager {
    pub fn new() -> NotificationManager {
        NotificationManager {
            notifications: HashMap::new(),
        }
    }

    pub async fn handle_notification_request(
        &mut self,
        client_id: &str,
        msg: NotificationRequest,
        client_manager: &ClientManager,
        subscription_manager: &SubscriptionManager,
    ) -> io::Result<()> {
        if msg.is_add {
            self.add_notification(
                client_id,
                msg.pattern.as_str(),
                client_manager,
                subscription_manager,
            )
            .await
        } else {
            self.remove_notification(client_id, msg.pattern.as_str(), false)
                .await
        }
    }

    pub async fn add_notification(
        &mut self,
        listener_id: &str,
        pattern: &str,
        client_manager: &ClientManager,
        subscription_manager: &SubscriptionManager,
    ) -> io::Result<()> {
        // Add or get the subscription.
        if !self.notifications.contains_key(pattern) {
            self.notifications
                .insert(pattern.to_owned(), Notification::new(pattern)?);
        }
        let notification = self.notifications.get_mut(pattern).unwrap();

        if let Some(count) = notification.listeners.get_mut(listener_id) {
            *count += 1;
        } else {
            notification.listeners.insert(listener_id.into(), 1);
        }

        for (topic, subscribers) in subscription_manager.find_subscriptions(&notification.regex) {
            if notification.regex.is_match(topic.as_str()) {
                for subscriber_id in &subscribers {
                    let client = client_manager.get(subscriber_id).ok_or(io::Error::new(
                        io::ErrorKind::Other,
                        format!("unknown client {subscriber_id}"),
                    ))?;
                    let message = ForwardedSubscriptionRequest {
                        client_id: subscriber_id.clone(),
                        host: client.host.clone(),
                        user: client.user.clone(),
                        topic: topic.clone(),
                        is_add: true,
                    };
                    let event =
                        ServerEvent::OnMessage(Message::ForwardedSubscriptionRequest(message));
                    client
                        .tx
                        .send(event)
                        .await
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
                }
            }
        }

        Ok(())
    }

    pub async fn remove_notification(
        &mut self,
        listener_id: &str,
        pattern: &str,
        is_listener_closed: bool,
    ) -> io::Result<()> {
        let Some(notification) = self.notifications.get_mut(pattern) else {
            return Ok(());
        };

        let Some(count) = notification.listeners.get_mut(listener_id) else {
            return Ok(());
        };

        if is_listener_closed {
            *count = 0;
        } else {
            *count -= 1;
        }

        if *count == 0 {
            notification.listeners.remove(listener_id);
            log::debug!("removed all notifications for {listener_id} on {pattern}")
        } else {
            log::debug!("removed one notification for {listener_id} on {pattern}")
        }

        if notification.listeners.len() == 0 {
            self.notifications.remove(pattern);
        }

        Ok(())
    }

    pub async fn notify_listeners(
        &self,
        subscriber_id: &str,
        topic: &str,
        is_add: bool,
        client_manager: &ClientManager,
    ) -> io::Result<()> {
        log::debug!(
            "notify_listeners: subscriber_id={subscriber_id}, topic={topic}, is_add={is_add}"
        );

        for (_pattern, notification) in &self.notifications {
            if notification.regex.is_match(topic) {
                let subscriber = client_manager.get(&subscriber_id).ok_or(io::Error::new(
                    io::ErrorKind::Other,
                    format!("unknown client {subscriber_id}"),
                ))?;

                let message = ForwardedSubscriptionRequest {
                    client_id: subscriber_id.into(),
                    host: subscriber.host.clone(),
                    user: subscriber.user.clone(),
                    topic: topic.to_string(),
                    is_add,
                };

                for listener_id in notification.listeners.keys() {
                    if let Some(listener) = client_manager.get(listener_id) {
                        let event = ServerEvent::OnMessage(Message::ForwardedSubscriptionRequest(
                            message.clone(),
                        ));

                        listener
                            .tx
                            .send(event)
                            .await
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn handle_close(&mut self, listener_id: &str) -> io::Result<()> {
        let patterns = self.find_listener_patterns(listener_id);
        for pattern in patterns {
            self.remove_notification(listener_id, &pattern, true)
                .await?
        }
        Ok(())
    }

    fn find_listener_patterns(&self, listener_id: &str) -> Vec<String> {
        let mut patterns: Vec<String> = Vec::new();
        for (pattern, notification) in &self.notifications {
            if notification.listeners.contains_key(listener_id) {
                patterns.push(pattern.clone());
            }
        }
        patterns
    }
}
