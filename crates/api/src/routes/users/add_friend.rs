use rocket::State;
use rocket::serde::json::Json;
use sonm_database::util::reference::Reference;
use sonm_database::{AMQP, Database, User};
use sonm_models::v0;
use sonm_result::{Result, create_error};

/// # Accept Friend Request
///
/// Accept another user's friend request.
#[openapi(tag = "Relationships")]
#[put("/<target>/friend")]
pub async fn add(
    db: &State<Database>,
    amqp: &State<AMQP>,
    mut user: User,
    target: Reference<'_>,
) -> Result<Json<v0::User>> {
    let mut target = target.as_user(db).await?;

    if user.bot.is_some() || target.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    user.add_friend(db, amqp, &mut target).await?;
    Ok(Json(target.into(db, &user).await))
}
