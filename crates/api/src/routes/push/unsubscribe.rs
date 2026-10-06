use rocket_empty::EmptyResponse;
use sonm_database::{Database, Session};
use sonm_result::{Result, create_database_error};

use rocket::State;

/// # Unsubscribe
///
/// Remove the Web Push subscription associated with the current session.
#[openapi(tag = "Web Push")]
#[post("/unsubscribe")]
pub async fn unsubscribe(db: &State<Database>, mut session: Session) -> Result<EmptyResponse> {
    session.subscription = None;
    session
        .save(db)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| create_database_error!("save", "session"))
}
