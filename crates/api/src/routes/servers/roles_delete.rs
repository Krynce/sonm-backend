use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{
    AuditLogEntryAction, Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    voice::VoiceClient,
};
use sonm_permissions::{ChannelPermission, calculate_server_permissions};
use sonm_result::{Result, create_error};

use crate::util::{audit_log_reason::AuditLogReason, voice::sync_server_voice_permissions};

/// # Delete Role
///
/// Delete a server role by its id.
#[openapi(tag = "Server Permissions")]
#[delete("/<target>/roles/<role_id>")]
pub async fn delete(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    role_id: String,
    voice_client: &State<VoiceClient>,
) -> Result<EmptyResponse> {
    let mut server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageRole)?;

    let member_rank = query.get_member_rank().unwrap_or(i64::MIN);

    let role = server
        .roles
        .remove(&role_id)
        .ok_or_else(|| create_error!(NotFound))?;

    if role.rank <= member_rank {
        return Err(create_error!(NotElevated));
    }

    if role.is_managed() {
        return Err(create_error!(InvalidOperation));
    }

    role.delete(db, &server.id).await?;

    AuditLogEntryAction::RoleDelete {
        role: role_id.clone(),
        name: role.name,
    }
    .insert(db, server.id.clone(), reason, user.id, None)
    .await;

    sync_server_voice_permissions(db, voice_client, &server, Some(&role_id)).await?;

    Ok(EmptyResponse)
}
