use rocket::{State, serde::json::Json};
use sonm_database::{Database, User, util::reference::Reference};
use sonm_models::v0;
use sonm_permissions::ChannelPermission;
use sonm_result::Result;

use crate::util::checks::require_server_permission;

/// # Fetch Invites
///
/// Fetch all server invites.
#[openapi(tag = "Server Members")]
#[get("/<target>/invites")]
pub async fn invites(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
) -> Result<Json<Vec<v0::Invite>>> {
    let server = target.as_server(db).await?;
    require_server_permission(db, &user, &server, ChannelPermission::ManageServer).await?;

    db.fetch_invites_for_server(&server.id)
        .await
        .map(|v| v.into_iter().map(Into::into).collect())
        .map(Json)
}
