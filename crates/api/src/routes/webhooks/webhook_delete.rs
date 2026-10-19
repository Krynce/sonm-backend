use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{AuditLogEntryAction, Database, User, util::reference::Reference};
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use crate::util::audit_log_reason::AuditLogReason;

use crate::util::checks::require_channel_permission;

/// # Deletes a webhook
///
/// Deletes a webhook
#[openapi(tag = "Webhooks")]
#[delete("/<webhook_id>")]
pub async fn webhook_delete(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    webhook_id: Reference<'_>,
) -> Result<EmptyResponse> {
    let webhook = webhook_id.as_webhook(db).await?;
    let channel = db.fetch_channel(&webhook.channel_id).await?;

    require_channel_permission(db, &user, &channel, ChannelPermission::ManageWebhooks).await?;

    webhook.delete(db).await?;

    AuditLogEntryAction::WebhookDelete {
        webhook: webhook.id,
        name: webhook.name,
        channel: webhook.channel_id,
    }
    .insert(
        db,
        channel
            .server()
            .expect("Webhook created on non server channel")
            .to_string(),
        reason,
        user.id,
        Some(webhook.creator_id),
    )
    .await;

    Ok(EmptyResponse)
}
