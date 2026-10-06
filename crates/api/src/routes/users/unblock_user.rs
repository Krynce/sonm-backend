use rocket::State;
use rocket::serde::json::Json;
use sonm_database::util::reference::Reference;
use sonm_database::{Database, User};
use sonm_models::v0;
use sonm_result::{Result, create_error};

/// # Unblock User
///
/// Unblock another user by their id.
#[openapi(tag = "Relationships")]
#[delete("/<target>/block")]
pub async fn unblock(
    db: &State<Database>,
    mut user: User,
    target: Reference<'_>,
) -> Result<Json<v0::User>> {
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    let mut target = target.as_user(db).await?;

    user.unblock_user(db, &mut target).await?;
    Ok(Json(target.into(db, &user).await))
}
