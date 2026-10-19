use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{AuditLogEntryAction, Database, Invite, User, util::reference::Reference};
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use crate::util::audit_log_reason::AuditLogReason;

use crate::util::checks::require_server_permission;

/// # Delete Invite
///
/// Delete an invite by its id.
#[openapi(tag = "Invites")]
#[delete("/<target>")]
pub async fn delete(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
) -> Result<EmptyResponse> {
    let invite = target.as_invite(db).await?;

    if user.id == invite.creator() {
        db.delete_invite(invite.code()).await?;
    } else {
        match invite {
            Invite::Server {
                code,
                server,
                channel,
                creator,
                ..
            } => {
                let server = db.fetch_server(&server).await?;
                require_server_permission(db, &user, &server, ChannelPermission::ManageServer)
                    .await?;

                db.delete_invite(&code).await?;

                AuditLogEntryAction::InviteDelete {
                    invite: code,
                    channel,
                }
                .insert(db, server.id, reason, user.id, Some(creator))
                .await;
            }
            _ => unreachable!(),
        }
    }

    Ok(EmptyResponse)
}
