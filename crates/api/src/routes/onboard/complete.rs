use sonm_database::{Database, Session, User};
use sonm_models::v0;
use sonm_result::{Result, create_error};

use rocket::{State, serde::json::Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::util::checks::validate;

/// # New User Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataOnboard {
    /// New username which will be used to identify the user on the platform
    #[validate(length(min = 2, max = 32), regex = "v0::RE_USERNAME")]
    username: String,
}

/// # Complete Onboarding
///
/// This sets a new username, completes onboarding and allows a user to start using Sonm.
#[openapi(tag = "Onboarding")]
#[post("/complete", data = "<data>")]
pub async fn complete(
    db: &State<Database>,
    session: Session,
    user: Option<User>,
    data: Json<DataOnboard>,
) -> Result<Json<v0::User>> {
    if user.is_some() {
        return Err(create_error!(AlreadyOnboarded));
    }

    let data = data.into_inner();
    validate(&data)?;

    Ok(Json(
        User::create(db, data.username, session.user_id, None)
            .await?
            .into_self(false)
            .await,
    ))
}
