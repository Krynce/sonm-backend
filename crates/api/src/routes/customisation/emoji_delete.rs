use sonm_database::{AuditLogEntryAction, Database, EmojiParent, User, util::reference::Reference};
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use rocket::State;
use rocket_empty::EmptyResponse;

use crate::util::audit_log_reason::AuditLogReason;

use crate::util::checks::require_server_permission;

/// # Delete Emoji
///
/// Delete an emoji by its id.
#[openapi(tag = "Emojis")]
#[delete("/emoji/<emoji_id>")]
pub async fn delete_emoji(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    emoji_id: Reference<'_>,
) -> Result<EmptyResponse> {
    // Fetch the emoji
    let emoji = emoji_id.as_emoji(db).await?;

    // If we uploaded the emoji, then we have permission to delete it
    if emoji.creator_id != user.id {
        // Otherwise, validate we have permission to delete from parent
        match &emoji.parent {
            EmojiParent::Server { id } => {
                let server = db.fetch_server(id.as_str()).await?;

                // Check for permission
                require_server_permission(
                    db,
                    &user,
                    &server,
                    ChannelPermission::ManageCustomisation,
                )
                .await?;
            }
            EmojiParent::Detached => return Ok(EmptyResponse),
        };
    }

    // Delete the emoji
    emoji.delete(db).await?;

    if let EmojiParent::Server { id: server_id } = emoji.parent {
        AuditLogEntryAction::EmojiDelete {
            emoji: emoji.id,
            name: emoji.name,
        }
        .insert(db, server_id, reason, user.id, Some(emoji.creator_id))
        .await;
    };

    Ok(EmptyResponse)
}
