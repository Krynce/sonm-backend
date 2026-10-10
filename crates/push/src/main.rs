#[macro_use]
extern crate log;

use std::sync::Arc;

use lapin::{
    Channel, Connection, ConnectionProperties,
    options::{
        BasicConsumeOptions, BasicQosOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::{AMQPValue, FieldTable},
};
use sonm_config::{Settings, config};
use sonm_database::{AMQP, Database};
use tokio::signal::ctrl_c;

mod consumers;
mod utils;
use consumers::{
    inbound::{
        dm_call::DmCallConsumer, fr_accepted::FRAcceptedConsumer, fr_received::FRReceivedConsumer,
        generic::GenericConsumer, mass_mention::MassMessageConsumer, message::MessageConsumer,
    },
    outbound::vapid::VapidOutboundConsumer,
};

use crate::utils::{Consumer, Delegate};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    // Configure logging and environment
    sonm_config::configure!(push);

    // Setup database
    let db = sonm_database::DatabaseInfo::Auto.connect().await.unwrap();

    let config = config().await;

    let connection = Arc::new(
        Connection::connect(
            &format!(
                "amqp://{}:{}@{}:{}",
                &config.rabbit.username,
                &config.rabbit.password,
                &config.rabbit.host,
                &config.rabbit.port,
            ),
            ConnectionProperties::default(),
        )
        .await
        .expect("Failed to connect to RabbitMQ"),
    );

    // Consumers go silent without a word if the broker restarts; exit so Docker rebuilds them.
    AMQP::watch_connection(connection.clone());

    let mut channels = Vec::new();

    // An explainer of how this works:
    // The inbound connections are on separate routing keys, such that they only receive the proper payload
    // from their respective api (prod or test).
    // However, the outbound queues that go to the services are routed to receive from both, so that messages
    // sent from beta are still notified on prod, and vice versa.

    // This'll require some interesting shimming if we need to add more events once this is in prod (different payloads between prod and test),
    // but that sounds like a problem for future us.

    channels.push(
        make_queue_and_consume::<GenericConsumer>(
            &db,
            &connection,
            &config,
            &config.push.generic_queue,
            &config.push.get_generic_routing_key(),
            None,
        )
        .await,
    );

    channels.push(
        make_queue_and_consume::<MessageConsumer>(
            &db,
            &connection,
            &config,
            &config.push.message_queue,
            &config.push.get_message_routing_key(),
            None,
        )
        .await,
    );

    channels.push(
        make_queue_and_consume::<FRReceivedConsumer>(
            &db,
            &connection,
            &config,
            &config.push.fr_received_queue,
            &config.push.get_fr_received_routing_key(),
            None,
        )
        .await,
    );

    channels.push(
        make_queue_and_consume::<FRAcceptedConsumer>(
            &db,
            &connection,
            &config,
            &config.push.fr_accepted_queue,
            &config.push.get_fr_accepted_routing_key(),
            None,
        )
        .await,
    );

    channels.push(
        make_queue_and_consume::<MassMessageConsumer>(
            &db,
            &connection,
            &config,
            &config.push.mass_mention_queue,
            &config.push.get_mass_mention_routing_key(),
            None,
        )
        .await,
    );

    channels.push(
        make_queue_and_consume::<DmCallConsumer>(
            &db,
            &connection,
            &config,
            &config.push.dm_call_queue,
            &config.push.get_dm_call_routing_key(),
            None,
        )
        .await,
    );

    if !config.push.vapid.public_key.is_empty() {
        channels.push(
            make_queue_and_consume::<VapidOutboundConsumer>(
                &db,
                &connection,
                &config,
                &config.push.vapid.queue,
                &config.push.vapid.queue,
                None,
            )
            .await,
        );
    }

    ctrl_c().await.unwrap();

    for channel in channels {
        let _ = channel.close(0, "close".into()).await;
    }
}

async fn make_queue_and_consume<F>(
    db: &Database,
    connection: &Arc<Connection>,
    config: &Settings,
    queue_name: &str,
    routing_key: &str,
    queue_args: Option<FieldTable>,
) -> Arc<Channel>
where
    F: Consumer,
{
    let channel = Arc::new(connection.create_channel().await.unwrap());

    channel
        .exchange_declare(
            config.push.exchange.clone().into(),
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare exchange");

    let mut queue_name = queue_name.to_string();

    if config.push.production {
        queue_name += "-prd";
    } else {
        queue_name += "-tst";
    }

    let queue_name = queue_name.as_str();

    let args = QueueDeclareOptions {
        durable: true,
        ..Default::default()
    };

    // Failed deliveries are parked on a single dead-letter queue instead of being dropped.
    let dlx = format!("{}-dlx", config.push.exchange);
    let dlq = format!("{}-dlq", config.push.exchange);

    channel
        .exchange_declare(
            dlx.as_str().into(),
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare dead-letter exchange");

    channel
        .queue_declare(dlq.as_str().into(), args, FieldTable::default())
        .await
        .expect("Failed to declare dead-letter queue");

    channel
        .queue_bind(
            dlq.as_str().into(),
            dlx.as_str().into(),
            "".into(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to bind dead-letter queue");

    let mut declare_args = queue_args.unwrap_or_default();
    declare_args.insert(
        "x-dead-letter-exchange".into(),
        AMQPValue::LongString(dlx.as_str().into()),
    );

    channel
        .queue_declare(queue_name.into(), args, declare_args)
        .await
        .unwrap();

    // Cap unacknowledged deliveries so they cannot pile up in memory.
    channel
        .basic_qos(20, BasicQosOptions::default())
        .await
        .expect("Failed to set prefetch");

    channel
        .queue_bind(
            queue_name.into(),
            config.push.exchange.clone().into(),
            routing_key.into(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("This probably means the notifications exchange does not exist in rabbitmq!");

    let consumer = channel
        .basic_consume(
            queue_name.into(),
            "".into(),
            BasicConsumeOptions {
                no_ack: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .unwrap();
    info!(
        "Consuming routing key {} as queue {}, tag {}",
        routing_key,
        queue_name,
        consumer.tag()
    );

    let delegate = Delegate(F::create(db.clone(), connection.clone(), channel.clone()).await);

    consumer.set_delegate(delegate);

    channel
}
