use rocket::serde::json::Json;
use sonm_database::User;
use sonm_models::v0;
use sonm_result::Result;

/// # Fetch Self
///
/// Retrieve your user information.
#[openapi(tag = "User Information")]
#[get("/@me")]
pub async fn fetch(user: User) -> Result<Json<v0::User>> {
    Ok(Json(user.into_self(false).await))
}
