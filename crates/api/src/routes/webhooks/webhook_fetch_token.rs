use rocket::{State, serde::json::Json};
use sonm_database::{Database, util::reference::Reference};
use sonm_models::v0::Webhook;
use sonm_result::Result;

/// # Gets a webhook
///
/// Gets a webhook with a token
#[openapi(tag = "Webhooks")]
#[get("/<webhook_id>/<token>")]
pub async fn webhook_fetch_token(
    db: &State<Database>,
    webhook_id: Reference<'_>,
    token: String,
) -> Result<Json<Webhook>> {
    let webhook = webhook_id.as_webhook(db).await?;
    webhook.assert_token(&token)?;
    Ok(Json(webhook.into()))
}
