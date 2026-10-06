use sonm_database::AuditLogEntryAction;
use sonm_database::util::permissions::DatabasePermissionQuery;
use sonm_database::{Channel, Database, User, util::reference::Reference};
use sonm_models::v0;
use sonm_permissions::{ChannelPermission, calculate_server_permissions};
use sonm_result::{Result, create_error};

use rocket::State;
use rocket::serde::json::Json;
use validator::Validate;

use crate::util::audit_log_reason::AuditLogReason;

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
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let mut server = server.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageChannel)?;

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
