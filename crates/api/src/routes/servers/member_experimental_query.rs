use std::collections::HashMap;

use futures::{future::join_all, try_join};
use rocket::{State, serde::json::Json};
use serde::{Deserialize, Serialize};
use sonm_database::{
    Database, Member, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_models::v0;
use sonm_permissions::PermissionQuery;
use sonm_result::{Result, create_error};

/// How many candidates each half of the search may return before we rank them
const CANDIDATE_LIMIT: i64 = 50;

/// # Query Parameters
#[derive(Deserialize, JsonSchema, FromForm)]
pub struct OptionsQueryMembers {
    /// String to search for
    query: String,

    /// Discourage use of this API
    experimental_api: bool,
}

/// # Query members by name
#[derive(Serialize, JsonSchema)]
pub struct MemberQueryResponse {
    /// List of members
    members: Vec<v0::Member>,
    /// List of users
    users: Vec<v0::User>,
}

/// # Query members by name
///
/// Query members by a given name, this API is not stable and will be removed in the future.
#[openapi(tag = "Server Members")]
#[get("/<target>/members_experimental_query?<options..>")]
pub async fn member_experimental_query(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    options: OptionsQueryMembers,
) -> Result<Json<MemberQueryResponse>> {
    if !options.experimental_api {
        return Err(create_error!(InternalError));
    }

    let server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    if !query.are_we_a_member().await {
        return Err(create_error!(NotFound));
    }

    // A member matches on their nickname, or — when they have none — on their username.
    // Both halves are matched and capped by MongoDB: fetching every member of the server
    // to filter in Rust meant reading the whole collection to return ten rows.
    let (nickname_matches, username_matches) = try_join!(
        db.fetch_members_by_nickname(&server.id, &options.query, CANDIDATE_LIMIT),
        db.fetch_users_by_username(&options.query, CANDIDATE_LIMIT)
    )?;

    let username_match_ids: Vec<String> = username_matches
        .iter()
        .map(|user| user.id.clone())
        .collect();

    let mut candidates: HashMap<String, Member> = nickname_matches
        .into_iter()
        .map(|member| (member.id.user.clone(), member))
        .collect();

    for member in db.fetch_members(&server.id, &username_match_ids).await? {
        // A nickname shadows the username for matching purposes
        if member.nickname.is_none() {
            candidates.entry(member.id.user.clone()).or_insert(member);
        }
    }

    let mut members: Vec<Member> = candidates.into_values().collect();
    let mut users = db
        .fetch_users(
            &members
                .iter()
                .map(|member| member.id.user.clone())
                .collect::<Vec<String>>(),
        )
        .await?;

    // Ensure the lists match up exactly
    members.sort_by(|a, b| a.id.user.cmp(&b.id.user));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    let mut zipped_vec: Vec<(Member, User)> = members.into_iter().zip(users).collect();

    // Sort remaining matches by length
    zipped_vec.sort_by(|(member_a, user_a), (member_b, user_b)| {
        let left = member_a.nickname.as_ref().unwrap_or(&user_a.username);
        let right = member_b.nickname.as_ref().unwrap_or(&user_b.username);
        left.len().cmp(&right.len())
    });

    // Take the first ten and return them
    let (members, users): (Vec<Member>, Vec<User>) = zipped_vec.into_iter().take(10).unzip();
    Ok(Json(MemberQueryResponse {
        members: members.into_iter().map(Into::into).collect(),
        users: join_all(
            users
                .into_iter()
                .map(|other_user| other_user.into(db, &user)),
        )
        .await,
    }))
}
