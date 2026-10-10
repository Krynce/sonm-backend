use std::{
    env,
    sync::atomic::{AtomicUsize, Ordering},
};

use sonm_database::AMQP;
use sonm_presence::clear_region;
use tokio::{io::AsyncWriteExt, net::TcpListener};

#[macro_use]
extern crate log;

pub mod config;
pub mod events;

mod database;
mod websocket;

/// Currently connected WebSocket clients
static CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// Answer any request with 200 and the connection count
///
/// The gateway speaks WebSocket on a raw socket, so the container healthcheck gets its own
/// port instead; a hand-written response avoids pulling an HTTP framework in for one route.
async fn serve_health(listener: TcpListener) {
    while let Ok((mut stream, _)) = listener.accept().await {
        let body = format!("{}\n", CONNECTIONS.load(Ordering::Relaxed));
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );

        tokio::spawn(async move {
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.shutdown().await;
        });
    }
}

#[tokio::main]
async fn main() {
    // Configure requirements for the gateway.
    sonm_config::configure!(gateway);
    database::connect().await;

    // Clean up the current region information.
    let no_clear_region = env::var("NO_CLEAR_PRESENCE").unwrap_or_else(|_| "0".into()) == "1";
    if !no_clear_region {
        clear_region(None).await;
    }

    AMQP::new_auto().await;

    // Setup a TCP listener to accept WebSocket connections on.
    // By default, we bind to port 14703 on all interfaces.
    let bind = env::var("HOST").unwrap_or_else(|_| "0.0.0.0:14703".into());
    info!("Listening on host {bind}");
    let try_socket = TcpListener::bind(bind).await;
    let listener = try_socket.expect("Failed to bind");

    // Health endpoint on its own port, for the container healthcheck.
    let health_bind = env::var("HEALTH_HOST").unwrap_or_else(|_| "0.0.0.0:14713".into());
    match TcpListener::bind(&health_bind).await {
        Ok(health_listener) => {
            info!("Serving health on {health_bind}");
            tokio::spawn(serve_health(health_listener));
        }
        Err(error) => error!("Failed to bind health endpoint on {health_bind}: {error:?}"),
    }

    // Start accepting new connections and spawn a client for each connection.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::task::spawn(async move {
            info!("User connected from {addr:?}");
            CONNECTIONS.fetch_add(1, Ordering::Relaxed);
            websocket::client(database::get_db(), stream, addr).await;
            CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
            info!("User disconnected from {addr:?}");
        });
    }
}
