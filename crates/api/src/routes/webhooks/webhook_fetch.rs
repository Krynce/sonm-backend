use rocket::{State, serde::json::Json};
use sonm_database::{Database, User, util::reference::Reference};
use sonm_models::v0::{ResponseWebhook, Webhook};
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use crate::util::checks::require_channel_permission;

/// # Gets a webhook
///
/// Gets a webhook
#[openapi(tag = "Webhooks")]
#[get("/<webhook_id>")]
pub async fn webhook_fetch(
    db: &State<Database>,
    webhook_id: Reference<'_>,
    user: User,
) -> Result<Json<ResponseWebhook>> {
    let webhook = webhook_id.as_webhook(db).await?;
    let channel = db.fetch_channel(&webhook.channel_id).await?;

    require_channel_permission(db, &user, &channel, ChannelPermission::ViewChannel).await?;

    Ok(Json(std::convert::Into::<Webhook>::into(webhook).into()))
}
