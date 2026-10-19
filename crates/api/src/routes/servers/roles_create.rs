use rocket::{State, serde::json::Json};
use sonm_config::config;
use sonm_database::{AuditLogEntryAction, Database, Role, User, util::reference::Reference};
use sonm_models::v0;
use sonm_permissions::ChannelPermission;
use sonm_result::{Result, create_error};

use crate::util::audit_log_reason::AuditLogReason;

use crate::util::checks::{require_server_permission, validate};

/// # Create Role
///
/// Creates a new server role.
#[openapi(tag = "Server Permissions")]
#[post("/<target>/roles", data = "<data>")]
pub async fn create(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    data: Json<v0::DataCreateRole>,
) -> Result<Json<v0::NewRoleResponse>> {
    let data = data.into_inner();
    validate(&data)?;

    let server = target.as_server(db).await?;
    require_server_permission(db, &user, &server, ChannelPermission::ManageRole).await?;

    let config = config().await;
    if server.roles.len() >= config.features.limits.global.server_roles {
        return Err(create_error!(TooManyRoles {
            max: config.features.limits.global.server_roles,
        }));
    };

    let role = Role::create(db, &server, data.name).await?;

    AuditLogEntryAction::RoleCreate {
        role: role.id.clone(),
        name: role.name.clone(),
    }
    .insert(db, server.id, reason, user.id, None)
    .await;

    Ok(Json(v0::NewRoleResponse {
        id: role.id.clone(),
        role: role.into(),
    }))
}
