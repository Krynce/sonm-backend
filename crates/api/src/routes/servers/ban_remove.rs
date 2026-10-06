use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{
    AuditLogEntryAction, Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_permissions::{ChannelPermission, calculate_server_permissions};
use sonm_result::Result;

use crate::util::audit_log_reason::AuditLogReason;

/// # Unban user
///
/// Remove a user's ban.
#[openapi(tag = "Server Members")]
#[delete("/<server>/bans/<target>")]
pub async fn unban(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    server: Reference<'_>,
    target: Reference<'_>,
) -> Result<EmptyResponse> {
    let server = server.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::BanMembers)?;

    let ban = target.as_ban(db, &server.id).await?;
    db.delete_ban(&ban.id).await?;

    AuditLogEntryAction::BanDelete {
        user: target.id.to_string(),
    }
    .insert(db, server.id, reason, user.id, Some(target.id.to_string()))
    .await;

    Ok(EmptyResponse)
}
