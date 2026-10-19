use rocket::{State, serde::json::Json};
use sonm_database::{Database, User, util::reference::Reference};
use sonm_models::v0::Webhook;
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use crate::util::checks::require_channel_permission;

/// # Gets all webhooks
///
/// Gets all webhooks inside the channel
#[openapi(tag = "Webhooks")]
#[get("/<channel_id>/webhooks")]
pub async fn fetch_webhooks(
    db: &State<Database>,
    user: User,
    channel_id: Reference<'_>,
) -> Result<Json<Vec<Webhook>>> {
    let channel = channel_id.as_channel(db).await?;

    require_channel_permission(db, &user, &channel, ChannelPermission::ManageWebhooks).await?;

    Ok(Json(
        db.fetch_webhooks_for_channel(channel.id())
            .await?
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<Webhook>>(),
    ))
}
