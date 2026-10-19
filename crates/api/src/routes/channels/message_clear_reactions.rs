use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{Database, PartialMessage, User, util::reference::Reference};
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use crate::util::checks::require_channel_permission;

/// # Remove All Reactions from Message
///
/// Remove your own, someone else's or all of a given reaction.
///
/// Requires `ManageMessages` permission.
#[openapi(tag = "Interactions")]
#[delete("/<target>/messages/<msg>/reactions")]
pub async fn clear_reactions(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    msg: Reference<'_>,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    require_channel_permission(db, &user, &channel, ChannelPermission::ManageMessages).await?;

    // Fetch relevant message
    let mut message = msg.as_message_in_channel(db, channel.id()).await?;

    // Clear reactions
    message
        .update(
            db,
            PartialMessage {
                reactions: Some(Default::default()),
                ..Default::default()
            },
            vec![],
        )
        .await
        .map(|_| EmptyResponse)
}
