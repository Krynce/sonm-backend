use rocket::{State, serde::json::Json};
use sonm_database::{Channel, Database, User, util::reference::Reference};
use sonm_models::v0;
use sonm_permissions::ChannelPermission;
use sonm_result::{Result, create_error};

use crate::util::checks::require_channel_permission;

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
    require_channel_permission(db, &user, &channel, ChannelPermission::ViewChannel).await?;

    if let Channel::Group { recipients, .. } = channel {
        Ok(Json(
            User::fetch_many_ids_as_mutuals(db, &user, &recipients).await?,
        ))
    } else {
        Err(create_error!(InvalidOperation))
    }
}
