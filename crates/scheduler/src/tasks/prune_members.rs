use std::time::Duration;

use log::warn;
use sonm_database::Database;
use sonm_result::Result;
use tokio::time::sleep;

pub async fn task(db: Database, _: sonm_database::AMQP) -> Result<()> {
    loop {
        let success = db.remove_dangling_members().await;
        if let Err(s) = success {
            sonm_config::capture_error(&s);
            warn!("Failed to prune dangling members: {:?}", &s);
        }

        sleep(Duration::from_secs(90)).await;
    }
}
