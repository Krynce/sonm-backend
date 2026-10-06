use sonm_config::config;
use sonm_database::{
    Database, DiscoverRequestStatus, DiscoverRequestType, User, util::reference::Reference,
};

use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_result::{Result, create_error};

/// # Delete Discover request
///
/// This cannot be used if your request is no longer in the queue (ie approved or rejected).
/// If you wish to reapply after a rejection, submit another POST.
/// This endpoint is ONLY USEFUL on instances running Discover .
#[openapi(tag = "Discover")]
#[delete("/<server>/discover")]
pub async fn discover_remove(
    db: &State<Database>,
    server: Reference<'_>,
    user: User,
) -> Result<EmptyResponse> {
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

    match ret.status {
        DiscoverRequestStatus::Approved(_) => Err(create_error!(ContactSupport {
            locale: "discover.server_removal_approved".to_string(),
            msg: "Contact support to have your server removed from Discover".to_string()
        })),
        DiscoverRequestStatus::Removed(_) => Err(create_error!(ContactSupport {
            locale: "discover.server_removal_removed".to_string(),
            msg: "Your server has been removed from discover, contact support for more information"
                .to_string()
        })),
        DiscoverRequestStatus::Denied(_) => Err(create_error!(NoEffect)),
        DiscoverRequestStatus::Pending | DiscoverRequestStatus::UnderReview => {
            db.delete_discover_request(ret.request_type, &ret.request_id)
                .await?;
            Ok(EmptyResponse)
        }
    }
}
