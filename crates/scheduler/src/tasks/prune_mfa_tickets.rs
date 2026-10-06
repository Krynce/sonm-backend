use std::time::Duration;

use sonm_database::Database;
use sonm_result::Result;
use tokio::time::sleep;

pub async fn task(db: Database) -> Result<()> {
    loop {
        let count = db.delete_expired_tickets().await?;
        log::info!("Pruned {count} expired MFA tickets");

        sleep(Duration::from_mins(5)).await
    }
}
