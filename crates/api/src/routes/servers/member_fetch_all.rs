use std::collections::HashSet;

use rocket::{State, serde::json::Json};
use sonm_database::{
    Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_models::v0;
use sonm_permissions::PermissionQuery;
use sonm_result::{Result, create_error};

/// # Fetch Members
///
/// Fetch all server members.
#[openapi(tag = "Server Members")]
#[get("/<target>/members?<options..>")]
pub async fn fetch_all(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    options: v0::OptionsFetchAllMembers,
) -> Result<Json<v0::AllMemberResponse>> {
    let server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    if !query.are_we_a_member().await {
        return Err(create_error!(NotFound));
    }

    let mut members = db.fetch_all_members(&server.id).await?;

    let user_ids: Vec<String> = members
        .iter()
        .map(|member| member.id.user.clone())
        .collect();

    let mut users = User::fetch_many_ids_as_mutuals(db, &user, &user_ids).await?;

    members.sort_by(|a, b| a.id.user.cmp(&b.id.user));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    // Ensure the lists match up exactly: `fetch_many_ids_as_mutuals` may return fewer users than
    // there are members, and `exclude_offline` drops more.
    let exclude_offline = options.exclude_offline.unwrap_or_default();
    let keep: HashSet<String> = users
        .iter()
        .filter(|user| !exclude_offline || user.online)
        .map(|user| user.id.clone())
        .collect();

    members.retain(|member| keep.contains(&member.id.user));
    users.retain(|user| keep.contains(&user.id));

    Ok(Json(v0::AllMemberResponse {
        members: members.into_iter().map(Into::into).collect(),
        users,
    }))
}
