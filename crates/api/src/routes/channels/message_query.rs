use rocket::{State, serde::json::Json};
use sonm_database::{
    Database, Message, MessageFilter, MessageQuery, MessageTimePeriod, User,
    util::reference::Reference,
};
use sonm_models::v0::{self, MessageSort};
use sonm_permissions::ChannelPermission;
use sonm_result::{Result, create_error};

use crate::util::checks::{require_channel_permission, validate};

/// # Fetch Messages
///
/// Fetch multiple messages.
#[openapi(tag = "Messaging")]
#[get("/<target>/messages?<options..>")]
pub async fn query(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    options: v0::OptionsQueryMessages,
) -> Result<Json<v0::BulkMessageResponse>> {
    validate(&options)?;

    if let Some(MessageSort::Relevance) = options.sort {
        return Err(create_error!(InvalidOperation));
    }

    let channel = target.as_channel(db).await?;

    require_channel_permission(db, &user, &channel, ChannelPermission::ReadMessageHistory).await?;

    let v0::OptionsQueryMessages {
        limit,
        before,
        after,
        sort,
        nearby,
        include_users,
    } = options;

    Message::fetch_with_users(
        db,
        MessageQuery {
            filter: MessageFilter {
                channel: Some(channel.id().to_string()),
                ..Default::default()
            },
            time_period: if let Some(nearby) = nearby {
                MessageTimePeriod::Relative { nearby }
            } else {
                MessageTimePeriod::Absolute {
                    before,
                    after,
                    sort,
                }
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
