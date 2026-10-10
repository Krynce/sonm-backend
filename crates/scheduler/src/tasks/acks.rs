use futures_lite::stream::StreamExt;
use lapin::{ExchangeKind, options::*, types::FieldTable};
use log::{debug, info};
use redis_kiss::{AsyncCommands, Conn as RedisConnection, get_connection};
use serde_json;
use sonm_config::config;
use sonm_database::{AMQP, Database, events::rabbit::AckEventPayload};
use sonm_result::{Result, ToSonmError};

pub async fn task(db: Database, amqp: AMQP) -> Result<()> {
    let config = config().await;

    let mut redis = get_connection()
        .await
        .expect("Failed to get redis connection");

    let reader_channel = amqp
        .connection()
        .create_channel()
        .await
        .expect("Failed to create channel");

    reader_channel
        .exchange_declare(
            config.rabbit.default_exchange.clone().into(),
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare exchange");

    reader_channel
        .queue_declare(
            config.rabbit.queues.acks.clone().into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind queue");

    reader_channel
        .queue_bind(
            config.rabbit.queues.acks.clone().into(),
            config.rabbit.default_exchange.into(),
            config.rabbit.queues.acks.clone().into(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind channel");

    let mut consumer = reader_channel
        .basic_consume(
            config.rabbit.queues.acks.into(),
            "scheduler-ack-consumer".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to create consumer");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            // Anything we cannot act on still has to be answered, or the prefetch slot leaks.
            let target = match serde_json::from_slice::<AckEventPayload>(&delivery.data) {
                Ok(payload) => {
                    debug!("Received ack event: {payload:?}");

                    match payload.channel_id {
                        Some(channel_id) => Some((payload.user_id, channel_id)),
                        None => {
                            sonm_config::capture_message(
                                "Received ack event without a channel id",
                                sonm_config::Level::Error,
                            );
                            None
                        }
                    }
                }
                Err(_) => {
                    sonm_config::capture_message(
                        format!("Failed to decode ack data: {:?}", delivery.data).as_str(),
                        sonm_config::Level::Error,
                    );
                    None
                }
            };

            let Some((user_id, channel_id)) = target else {
                _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                continue;
            };

            let redelivered = delivery.redelivered;

            if let Err(e) = process_channel_ack(&db, user_id, channel_id, &mut redis).await {
                sonm_config::capture_error(&e);

                // Give it one more go (likely a database blip), then drop it: this queue has
                // no dead-letter exchange.
                _ = delivery
                    .reject(BasicRejectOptions {
                        requeue: !redelivered,
                    })
                    .await;
            } else {
                _ = delivery.ack(BasicAckOptions { multiple: false }).await;
            }
        }
    }
    Ok(())
}

#[allow(clippy::disallowed_methods)]
async fn process_channel_ack(
    db: &Database,
    user: String,
    channel: String,
    redis: &mut RedisConnection,
) -> Result<()> {
    let message_id: Option<String> = redis
        .get_del(format!("acker:{user}+{channel}"))
        .await
        .to_internal_error()?;

    if let Some(message_id) = message_id {
        db.acknowledge_message(&channel, &user, &message_id).await?;

        info!("Set new state for ack: {}:{}:{}", channel, user, message_id);
    } else {
        // Normal race: the pending key was already consumed by another delivery.
        debug!("No pending ack for {channel}:{user}");
    }

    Ok(())
}
