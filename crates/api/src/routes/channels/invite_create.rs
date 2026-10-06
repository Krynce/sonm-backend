use iso8601_timestamp::{Duration, Timestamp};
use sonm_database::{
    AuditLogEntryAction, Database, Invite, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_models::v0;
use sonm_permissions::{ChannelPermission, calculate_channel_permissions};

use crate::util::audit_log_reason::AuditLogReason;
use rocket::{State, serde::json::Json};
use sonm_config::config;
use sonm_result::{Result, create_error};

/// # Create Invite
///
/// Creates an invite to this channel.
///
/// Channel must be a `TextChannel`.
#[openapi(tag = "Channel Invites")]
#[post("/<target>/invites", data = "<data>")]
pub async fn create_invite(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    data: Json<v0::DataCreateInvite>,
) -> Result<Json<v0::Invite>> {
    let data = data.into_inner();

    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    let max_invite_duration_days = Duration::days(
        config()
            .await
            .features
            .limits
            .global
            .max_invite_duration_days as i64,
    );

    if let Some(expires) = data.expires {
        let now = Timestamp::now_utc();
        if expires <= now || expires > now + max_invite_duration_days {
            return Err(create_error!(InvalidOperation));
        }
    }

    let channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::InviteOthers)?;

    let invite =
        Invite::create_channel_invite(db, &user, &channel, data.max_uses, data.expires).await?;

    if let Some(server_id) = channel.server() {
        AuditLogEntryAction::InviteCreate {
            invite: invite.code().to_string(),
            channel: channel.id().to_string(),
        }
        .insert(db, server_id.to_string(), reason, user.id, None)
        .await;
    }

    Ok(Json(invite.into()))
}
