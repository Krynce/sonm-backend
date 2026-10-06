use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{
    AuditLogEntryAction, Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_permissions::{ChannelPermission, calculate_channel_permissions};
use sonm_result::Result;

use crate::util::audit_log_reason::AuditLogReason;

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

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageWebhooks)?;

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
