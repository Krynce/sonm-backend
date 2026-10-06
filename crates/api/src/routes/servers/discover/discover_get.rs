use sonm_config::config;
use sonm_database::{Database, DiscoverRequestType, User, util::reference::Reference};
use sonm_models::v0;

use rocket::{State, serde::json::Json};
use sonm_result::{Result, create_error};

/// # Get Discover request status
///
/// Fetches the status of your Discover request.
/// If it has been approved or denied, the reason will be provided (if applicable).
/// This endpoint is ONLY USEFUL on instances running Discover .
#[openapi(tag = "Discover")]
#[get("/<server>/discover")]
pub async fn discover_get(
    db: &State<Database>,
    server: Reference<'_>,
    user: User,
) -> Result<Json<v0::DiscoverRequest>> {
    let config = config().await;
    if !config.production {
        return Err(create_error!(NoEffect));
    }

    let server = server.as_server(db).await?;
    if server.owner != user.id && !user.privileged {
        return Err(create_error!(NotOwner));
    }

    if db
        .get_discover_ban(DiscoverRequestType::Server, &server.id)
        .await
        .is_ok()
    {
        return Err(create_error!(Banned));
    }

    let ret = db
        .fetch_discover_request_by_item_id(DiscoverRequestType::Server, &server.id)
        .await?;

    Ok(Json(ret.into()))
}
