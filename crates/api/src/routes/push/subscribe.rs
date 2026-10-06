use rocket::{State, serde::json::Json};
use rocket_empty::EmptyResponse;
use sonm_database::{Database, Session};
use sonm_models::v0;
use sonm_result::{Result, create_database_error};

/// # Push Subscribe
///
/// Create a new Web Push subscription.
///
/// If an existing subscription exists on this session, it will be removed.
#[openapi(tag = "Web Push")]
#[post("/subscribe", data = "<data>")]
pub async fn subscribe(
    db: &State<Database>,
    mut session: Session,
    data: Json<v0::WebPushSubscription>,
) -> Result<EmptyResponse> {
    session.subscription = Some(data.into_inner().into());
    session
        .save(db)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| create_database_error!("save", "session"))
}
