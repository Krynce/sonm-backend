use rocket::{State, serde::json::Json};
use sonm_database::{
    AuditLogEntryAction, Channel, Database, File, User, Webhook, util::reference::Reference,
};
use sonm_models::v0;
use sonm_permissions::{ChannelPermission, DEFAULT_WEBHOOK_PERMISSIONS};
use sonm_result::{Result, create_error};
use ulid::Ulid;

use crate::util::audit_log_reason::AuditLogReason;

use crate::util::checks::{require_channel_permission, validate};

/// # Creates a webhook
///
/// Creates a webhook which 3rd party platforms can use to send messages
#[openapi(tag = "Webhooks")]
#[post("/<channel_id>/webhooks", data = "<data>")]
pub async fn create_webhook(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    channel_id: Reference<'_>,
    data: Json<v0::CreateWebhookBody>,
) -> Result<Json<v0::Webhook>> {
    let data = data.into_inner();
    validate(&data)?;

    let channel = channel_id.as_channel(db).await?;

    if !matches!(channel, Channel::TextChannel { .. } | Channel::Group { .. }) {
        return Err(create_error!(InvalidOperation));
    }

    require_channel_permission(db, &user, &channel, ChannelPermission::ManageWebhooks).await?;

    let webhook_id = Ulid::new().to_string();

    let avatar = match &data.avatar {
        Some(id) => Some(File::use_webhook_avatar(db, id, &webhook_id, &user.id).await?),
        None => None,
    };

    let webhook = Webhook {
        id: webhook_id,
        name: data.name,
        avatar,
        creator_id: user.id.clone(),
        channel_id: channel.id().to_string(),
        permissions: *DEFAULT_WEBHOOK_PERMISSIONS,
        token: Some(nanoid::nanoid!(64)),
    };

    webhook.create(db).await?;

    if let Some(server_id) = channel.server() {
        AuditLogEntryAction::WebhookCreate {
            webhook: webhook.id.clone(),
            name: webhook.name.clone(),
            channel: webhook.channel_id.clone(),
        }
        .insert(db, server_id.to_string(), reason, user.id, None)
        .await;
    };

    Ok(Json(webhook.into()))
}
