use std::env;

use rocket::{Config, build, routes};
use sonm_database::DatabaseInfo;
use sonm_database::{AMQP, voice::VoiceClient};
use sonm_result::Result;
use std::net::Ipv4Addr;

mod api;
mod guard;

// `rocket::Error` is 224 bytes and `rocket::main` fixes the signature; it is returned once,
// at process exit, so boxing it buys nothing.
#[allow(clippy::result_large_err)]
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    sonm_config::configure!(voice);

    let amqp = AMQP::new_auto().await;

    let database = DatabaseInfo::Auto.connect().await.unwrap();
    let voice_client = VoiceClient::from_config().await;

    let _rocket = build()
        .manage(database)
        .manage(voice_client)
        .manage(amqp)
        .mount("/", routes![api::ingress, api::health])
        .configure(Config {
            port: 8500,
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            ..Default::default()
        })
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}
