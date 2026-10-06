use rocket::{State, serde::json::Json};
use sonm_database::{
    Channel, Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_models::v0;
use sonm_permissions::{ChannelPermission, calculate_channel_permissions};
use sonm_result::{Result, create_error};

/// # Fetch Group Members
///
/// Retrieves all users who are part of this group.
///
/// This may not return full user information if users are not friends but have mutual connections.
#[openapi(tag = "Groups")]
#[get("/<target>/members")]
pub async fn fetch_members(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
) -> Result<Json<Vec<v0::User>>> {
    let channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;

    if let Channel::Group { recipients, .. } = channel {
        Ok(Json(
            User::fetch_many_ids_as_mutuals(db, &user, &recipients).await?,
        ))
    } else {
        Err(create_error!(InvalidOperation))
    }
}
