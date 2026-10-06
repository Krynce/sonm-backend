use sonm_database::{Database, util::reference::Reference};
use sonm_models::v0;
use sonm_result::Result;

use rocket::{State, serde::json::Json};

/// # Fetch Emoji
///
/// Fetch an emoji by its id.
#[openapi(tag = "Emojis")]
#[get("/emoji/<emoji_id>")]
pub async fn fetch_emoji(db: &State<Database>, emoji_id: Reference<'_>) -> Result<Json<v0::Emoji>> {
    emoji_id
        .as_emoji(db)
        .await
        .map(|emoji| emoji.into())
        .map(Json)
}
