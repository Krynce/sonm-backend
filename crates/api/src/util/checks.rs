use sonm_database::{Channel, Database, Server, User, util::permissions::DatabasePermissionQuery};
use sonm_permissions::{
    ChannelPermission, calculate_channel_permissions, calculate_server_permissions,
};
use sonm_result::{Result, create_error};
use validator::Validate;

/// Fail with `MissingPermission` unless `user` has `permission` in `channel`
pub async fn require_channel_permission(
    db: &Database,
    user: &User,
    channel: &Channel,
    permission: ChannelPermission,
) -> Result<()> {
    let mut query = DatabasePermissionQuery::new(db, user).channel(channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(permission)
}

/// Fail with `MissingPermission` unless `user` has `permission` in `server`
pub async fn require_server_permission(
    db: &Database,
    user: &User,
    server: &Server,
    permission: ChannelPermission,
) -> Result<()> {
    let mut query = DatabasePermissionQuery::new(db, user).server(server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(permission)
}

/// Fail with `FailedValidation` if `data` does not pass its validator rules
pub fn validate(data: &impl Validate) -> Result<()> {
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })
}
