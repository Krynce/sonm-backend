use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{
    AuditLogEntryAction, Database, RemovalIntention, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    voice::{
        UserVoiceChannel, VoiceClient, get_user_voice_channel_in_server,
        remove_user_from_voice_channel,
    },
};
use sonm_permissions::{ChannelPermission, calculate_server_permissions};
use sonm_result::{Result, create_error};

use crate::util::audit_log_reason::AuditLogReason;

/// # Kick Member
///
/// Removes a member from the server.
#[openapi(tag = "Server Members")]
#[delete("/<server_id>/members/<member_id>")]
pub async fn kick(
    db: &State<Database>,
    voice_client: &State<VoiceClient>,
    user: User,
    reason: AuditLogReason,
    server_id: Reference<'_>,
    member_id: Reference<'_>,
) -> Result<EmptyResponse> {
    let server = server_id.as_server(db).await?;

    if member_id.id == user.id {
        return Err(create_error!(CannotRemoveYourself));
    }

    if member_id.id == server.owner {
        return Err(create_error!(InvalidOperation));
    }

    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::KickMembers)?;

    let member = member_id.as_member(db, &server.id).await?;
    if member.get_ranking(query.server_ref().as_ref().unwrap())
        <= query.get_member_rank().unwrap_or(i64::MIN)
    {
        return Err(create_error!(NotElevated));
    }

    member
        .remove(db, &server, RemovalIntention::Kick, false)
        .await?;

    server.cleanup_managed_bot_role(db, &member.id.user).await?;

    AuditLogEntryAction::MemberKick {
        user: member.id.user.clone(),
    }
    .insert(
        db,
        server.id.clone(),
        reason,
        user.id,
        Some(member.id.user.clone()),
    )
    .await;

    if let Some(channel_id) = get_user_voice_channel_in_server(member_id.id, &server.id).await? {
        remove_user_from_voice_channel(
            voice_client,
            &UserVoiceChannel {
                id: channel_id,
                server_id: Some(server.id.clone()),
            },
            member_id.id,
        )
        .await?;
    };

    Ok(EmptyResponse)
}
