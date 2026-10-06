use std::{collections::HashMap, sync::Arc};

use crate::utils::Consumer;
use anyhow::Result;
use async_trait::async_trait;
use lapin::{Channel, Connection, message::Delivery};
use log::debug;
use sonm_database::{Database, events::rabbit::*};

#[derive(Clone)]
#[allow(unused)]
pub struct FRAcceptedConsumer {
    db: Database,
    connection: Arc<Connection>,
    channel: Arc<Channel>,
}

#[async_trait]
impl Consumer for FRAcceptedConsumer {
    async fn create(db: Database, connection: Arc<Connection>, channel: Arc<Channel>) -> Self {
        Self {
            db,
            connection,
            channel,
        }
    }

    fn channel(&self) -> &Arc<Channel> {
        &self.channel
    }

    /// This consumer handles delegating messages into their respective platform queues.
    async fn consume(&self, delivery: Delivery) -> Result<()> {
        let payload: FRAcceptedPayload = serde_json::from_slice(&delivery.data)?;

        debug!("Received FR accept event");

        if let Ok(sessions) = self.db.fetch_sessions(&payload.user).await {
            let config = sonm_config::config().await;
            for session in sessions {
                if let Some(sub) = session.subscription {
                    let mut sendable = PayloadToService {
                        notification: PayloadKind::FRAccepted(payload.clone()),
                        token: sub.auth,
                        user_id: session.user_id,
                        session_id: session.id,
                        extras: HashMap::new(),
                    };

                    let routing_key = match sub.endpoint.as_str() {
                        // Native mobile subscriptions are not supported
                        "apn" | "fcm" => continue,
                        endpoint => {
                            sendable.extras.insert("p256dh".to_string(), sub.p256dh);
                            sendable
                                .extras
                                .insert("endpoint".to_string(), endpoint.to_string());

                            &config.push.vapid.queue
                        }
                    };

                    let payload = serde_json::to_string(&sendable)?;

                    self.publish_message(payload.as_bytes(), &config.push.exchange, routing_key)
                        .await?;
                }
            }
        }

        Ok(())
    }
}
