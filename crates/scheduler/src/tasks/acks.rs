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
            let payload = serde_json::from_slice::<AckEventPayload>(&delivery.data);

            if let Ok(payload) = payload {
                debug!("Received ack event: {payload:?}");

                if let Err(e) = process_channel_ack(
                    &db,
                    payload.user_id,
                    payload.channel_id.unwrap(),
                    &mut redis,
                )
                .await
                {
                    sonm_config::capture_error(&e);
                    _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                } else {
                    _ = delivery.ack(BasicAckOptions { multiple: false }).await;
                }
            } else {
                sonm_config::capture_message(
                    format!("Failed to decode ack data: {:?}", delivery.data).as_str(),
                    sonm_config::Level::Error,
                );
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

        Ok(())
    } else {
        Err(message_id.to_internal_error().expect_err("no err"))
    }
}
