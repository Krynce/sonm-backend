use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{AuditLogEntryAction, Database, User, util::reference::Reference};
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use crate::util::audit_log_reason::AuditLogReason;

use crate::util::checks::require_channel_permission;

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
        require_channel_permission(db, &user, &channel, ChannelPermission::ManageMessages).await?;

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
