use rocket::{State, serde::json::Json};
use sonm_database::{Database, User};
use sonm_models::v0;
use sonm_result::Result;

/// # Fetch Direct Message Channels
///
/// This fetches your direct messages, including any DM and group DM conversations.
#[openapi(tag = "Direct Messaging")]
#[get("/dms")]
pub async fn direct_messages(db: &State<Database>, user: User) -> Result<Json<Vec<v0::Channel>>> {
    db.find_direct_messages(&user.id)
        .await
        .map(|v| v.into_iter().map(Into::into).collect())
        .map(Json)
}
