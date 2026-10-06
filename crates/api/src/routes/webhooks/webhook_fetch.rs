use rocket::{State, serde::json::Json};
use sonm_database::{
    Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_models::v0::{ResponseWebhook, Webhook};
use sonm_permissions::{ChannelPermission, calculate_channel_permissions};
use sonm_result::Result;

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

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;

    Ok(Json(std::convert::Into::<Webhook>::into(webhook).into()))
}
