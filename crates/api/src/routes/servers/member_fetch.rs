use rocket::{State, serde::json::Json};
use sonm_database::{
    Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_models::v0;
use sonm_permissions::PermissionQuery;
use sonm_result::{Result, create_error};

/// # Fetch Member
///
/// Retrieve a member.
#[openapi(tag = "Server Members")]
#[get("/<server_id>/members/<member_id>?<roles>")]
pub async fn fetch(
    db: &State<Database>,
    user: User,
    server_id: Reference<'_>,
    member_id: Reference<'_>,
    roles: Option<bool>,
) -> Result<Json<v0::MemberResponse>> {
    let server = server_id.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    if !query.are_we_a_member().await {
        return Err(create_error!(NotFound));
    }

    let member = member_id.as_member(db, &server.id).await?;
    if let Some(true) = roles {
        Ok(Json(v0::MemberResponse::MemberWithRoles {
            roles: server
                .roles
                .into_iter()
                .filter(|(k, _)| member.roles.contains(k))
                .map(|(k, v)| (k, v.into()))
                .collect(),
            member: member.into(),
        }))
    } else {
        Ok(Json(v0::MemberResponse::Member(member.into())))
    }
}
