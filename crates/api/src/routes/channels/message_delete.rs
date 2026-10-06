use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{
    AuditLogEntryAction, Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_permissions::{ChannelPermission, calculate_channel_permissions};
use sonm_result::Result;

use crate::util::audit_log_reason::AuditLogReason;

/// # Delete Message
///
/// Delete a message you've sent or one you have permission to delete.
#[openapi(tag = "Messaging")]
#[delete("/<target>/messages/<msg>", rank = 2)]
pub async fn delete(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    msg: Reference<'_>,
) -> Result<EmptyResponse> {
    let message = msg.as_message_in_channel(db, target.id).await?;

    let channel = if message.author != user.id {
        let channel = target.as_channel(db).await?;
        let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
        calculate_channel_permissions(&mut query)
            .await
            .throw_if_lacking_channel_permission(ChannelPermission::ManageMessages)?;

        Some(channel)
    } else {
        None
    };

    message.delete(db).await?;

    if let Some(server) = channel.and_then(|c| c.server().map(|s| s.to_string())) {
        AuditLogEntryAction::MessageDelete {
            author: message.author.clone(),
            channel: message.channel.clone(),
        }
        .insert(
            db,
            server.to_string(),
            reason,
            user.id.clone(),
            Some(message.author),
        )
        .await;
    };

    Ok(EmptyResponse)
}
