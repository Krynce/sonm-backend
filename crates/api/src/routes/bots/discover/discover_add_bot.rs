use rocket_empty::EmptyResponse;
use sonm_config::config;
use sonm_database::{Database, DiscoverRequestType, User, util::reference::Reference};

use rocket::State;
use sonm_result::{Result, create_error};

/// # Add bot to Discover
///
/// This puts your bot into the Discover request queue.
/// This endpoint is ONLY USEFUL on instances running Discover .
#[openapi(tag = "Discover")]
#[put("/<bot_id>/discover")]
pub async fn discover_add_bot(
    db: &State<Database>,
    bot_id: Reference<'_>,
    user: User,
) -> Result<EmptyResponse> {
    let config = config().await;
    if !config.production {
        return Err(create_error!(NoEffect));
    }

    let bot = bot_id.as_bot(db).await?;
    if (bot.owner != user.id && bot.id != user.id) && !user.privileged {
        return Err(create_error!(NotOwner));
    }

    if db
        .get_discover_ban(DiscoverRequestType::Bot, &bot.id)
        .await
        .is_ok()
    {
        return Err(create_error!(Banned));
    }

    db.insert_discover_request(DiscoverRequestType::Bot, &bot.id)
        .await?;

    Ok(EmptyResponse)
}
