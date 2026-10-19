use sonm_database::AuditLogEntryAction;
use sonm_database::{Channel, Database, User, util::reference::Reference};
use sonm_models::v0;
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use rocket::State;
use rocket::serde::json::Json;

use crate::util::audit_log_reason::AuditLogReason;

use crate::util::checks::{require_server_permission, validate};

/// # Create Channel
///
/// Create a new Text or Voice channel.
#[openapi(tag = "Server Information")]
#[post("/<server>/channels", data = "<data>")]
pub async fn create_server_channel(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    server: Reference<'_>,
    data: Json<v0::DataCreateServerChannel>,
) -> Result<Json<v0::Channel>> {
    let data = data.into_inner();
    validate(&data)?;

    let mut server = server.as_server(db).await?;
    require_server_permission(db, &user, &server, ChannelPermission::ManageChannel).await?;

    let channel_name = data.name.clone();

    let channel = Channel::create_server_channel(db, &mut server, data, true).await?;

    AuditLogEntryAction::ChannelCreate {
        channel: channel.id().to_string(),
        name: channel_name,
    }
    .insert(db, server.id, reason, user.id, None)
    .await;

    Ok(Json(channel.into()))
}
