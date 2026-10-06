use rocket::{State, serde::json::Json};
use sonm_database::{
    Database, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_models::v0;
use sonm_permissions::{ChannelPermission, calculate_channel_permissions};
use sonm_result::{Result, create_error};

/// # Fetch Message
///
/// Retrieves a message by its id.
#[openapi(tag = "Messaging")]
#[get("/<target>/messages/<msg>")]
pub async fn fetch(
    db: &State<Database>,
    user: User,
    target: Reference<'_>,
    msg: Reference<'_>,
) -> Result<Json<v0::Message>> {
    let channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    let perms = calculate_channel_permissions(&mut query).await;

    perms.throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;
    perms.throw_if_lacking_channel_permission(ChannelPermission::ReadMessageHistory)?;

    let message = msg.as_message(db).await?;
    if message.channel != channel.id() {
        return Err(create_error!(NotFound));
    }

    Ok(Json(message.into_model(None, None)))
}
