use sonm_database::util::permissions::DatabasePermissionQuery;
use sonm_database::util::reference::Reference;
use sonm_database::{Database, User};
use sonm_models::v0;

use rocket::State;
use rocket::serde::json::Json;
use sonm_permissions::{UserPermission, calculate_user_permissions};
use sonm_result::{Result, create_error};

/// # Fetch Mutual Friends, Servers, Groups and DMs
///
/// Retrieve a list of mutual friends, servers, groups and DMs with another user.
#[openapi(tag = "Relationships")]
#[get("/<target>/mutual")]
pub async fn mutual(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
) -> Result<Json<v0::MutualResponse>> {
    if target.id == user.id {
        return Err(create_error!(InvalidOperation));
    }

    let target = target.as_user(db).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).user(&target);
    calculate_user_permissions(&mut query)
        .await
        .throw_if_lacking_user_permission(UserPermission::ViewProfile)?;

    Ok(Json(v0::MutualResponse {
        users: db.fetch_mutual_user_ids(&user.id, &target.id).await?,
        servers: db.fetch_mutual_server_ids(&user.id, &target.id).await?,
        channels: db.fetch_mutual_channel_ids(&user.id, &target.id).await?,
    }))
}
