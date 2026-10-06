use sonm_database::{
    AuditLogEntryAction, Database, Message, RemovalIntention, ServerBan, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    voice::{
        UserVoiceChannel, VoiceClient, get_user_voice_channel_in_server,
        remove_user_from_voice_channel,
    },
};
use sonm_models::v0;
use std::time::{Duration, SystemTime};

use rocket::{State, serde::json::Json};
use sonm_permissions::{ChannelPermission, calculate_server_permissions};
use sonm_result::{Result, create_error};
use validator::Validate;

use crate::util::audit_log_reason::AuditLogReason;

/// # Ban User
///
/// Ban a user by their id.
#[openapi(tag = "Server Members")]
#[put("/<server>/bans/<target>", data = "<data>")]
pub async fn ban(
    db: &State<Database>,
    voice_client: &State<VoiceClient>,
    user: User,
    audit_log_reason: AuditLogReason,
    server: Reference<'_>,
    target: Reference<'_>,
    data: Json<v0::DataBanCreate>,
) -> Result<Json<v0::ServerBan>> {
    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let server = server.as_server(db).await?;

    if target.id == user.id {
        return Err(create_error!(CannotRemoveYourself));
    }

    if target.id == server.owner {
        return Err(create_error!(InvalidOperation));
    }

    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::BanMembers)?;

    // If member exists, check privileges against them
    if let Ok(member) = target.as_member(db, &server.id).await {
        if member.get_ranking(query.server_ref().as_ref().unwrap())
            <= query.get_member_rank().unwrap_or(i64::MIN)
        {
            return Err(create_error!(NotElevated));
        }

        member
            .remove(db, &server, RemovalIntention::Ban, false)
            .await?;

        // If the member is in a voice channel while banned kick them from the voice channel
        if let Some(channel_id) = get_user_voice_channel_in_server(target.id, &server.id).await? {
            remove_user_from_voice_channel(
                voice_client,
                &UserVoiceChannel {
                    id: channel_id,
                    server_id: Some(server.id.clone()),
                },
                target.id,
            )
            .await?;
        }
    }
    // We do this outside the member check so we can sweep hit-and-run spammers who already left.
    if let Some(seconds) = data.delete_message_seconds {
        if seconds > 0 {
            let threshold_time = SystemTime::now() - Duration::from_secs(seconds as u64);

            Message::bulk_delete_by_author_since(db, &server.channels, target.id, threshold_time)
                .await?;
        }
    }

    let ban = ServerBan::create(db, &server, target.id, data.reason.clone()).await?;

    server.cleanup_managed_bot_role(db, target.id).await?;

    AuditLogEntryAction::BanCreate {
        user: target.id.to_string(),
    }
    .insert(
        db,
        server.id,
        audit_log_reason.0.or(data.reason),
        user.id,
        Some(target.id.to_string()),
    )
    .await;

    Ok(Json(ban.into()))
}
