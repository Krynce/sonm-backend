use rocket::{State, serde::json::Json};
use serde::{Deserialize, Serialize};
use sonm_database::{Account, Database, User};
use sonm_models::v0;
use sonm_result::{Result, create_error};
use validator::Validate;

use crate::util::checks::validate;

/// # Username Information
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataChangeUsername {
    /// New username
    #[validate(length(min = 2, max = 32), regex(path = "v0::RE_USERNAME"))]
    username: String,
    /// Current account password
    #[validate(length(min = 8, max = 1024))]
    password: String,
}

/// # Change Username
///
/// Change your username.
#[openapi(tag = "User Information")]
#[patch("/@me/username", data = "<data>")]
pub async fn change_username(
    db: &State<Database>,
    account: Account,
    mut user: User,
    data: Json<DataChangeUsername>,
) -> Result<Json<v0::User>> {
    let data = data.into_inner();
    validate(&data)?;

    account
        .verify_password(&data.password)
        .await
        .map_err(|_| create_error!(InvalidCredentials))?;

    user.update_username(db, data.username).await?;
    Ok(Json(user.into(db, None).await))
}
