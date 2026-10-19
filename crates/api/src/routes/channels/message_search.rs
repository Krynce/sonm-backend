use rocket::{State, serde::json::Json};
use sonm_database::{
    Database, Message, MessageFilter, MessageQuery, MessageTimePeriod, User,
    util::reference::Reference,
};
use sonm_models::v0;
use sonm_permissions::ChannelPermission;
use sonm_result::{Result, create_error};

use crate::util::checks::{require_channel_permission, validate};

/// # Search for Messages
///
/// This route searches for messages within the given parameters.
#[openapi(tag = "Messaging")]
#[post("/<target>/search", data = "<options>")]
pub async fn search(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    options: Json<v0::DataMessageSearch>,
) -> Result<Json<v0::BulkMessageResponse>> {
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    let options = options.into_inner();
    validate(&options)?;

    if options.query.is_some() && options.pinned.is_some() {
        return Err(create_error!(InvalidOperation));
    }

    let channel = target.as_channel(db).await?;

    require_channel_permission(db, &user, &channel, ChannelPermission::ReadMessageHistory).await?;

    let v0::DataMessageSearch {
        query,
        pinned,
        limit,
        before,
        after,
        sort,
        include_users,
    } = options;

    Message::fetch_with_users(
        db,
        MessageQuery {
            filter: MessageFilter {
                channel: Some(channel.id().to_string()),
                query,
                pinned,
                ..Default::default()
            },
            time_period: MessageTimePeriod::Absolute {
                before,
                after,
                sort: Some(sort),
            },
            limit,
        },
        &user,
        include_users,
        channel.server(),
    )
    .await
    .map(Json)
}
