use std::time::Duration;

use rocket::{State, serde::json::Json};
use rocket_empty::EmptyResponse;
use sonm_database::{
    AuditLogEntryAction, Database, Message, User,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
};
use sonm_models::v0;
use sonm_permissions::{ChannelPermission, calculate_channel_permissions};
use sonm_result::{Result, create_error};
use validator::Validate;

use crate::util::audit_log_reason::AuditLogReason;

/// # Bulk Delete Messages
///
/// Delete multiple messages you've sent or one you have permission to delete.
///
/// This will always require `ManageMessages` permission regardless of whether you own the message or not.
///
/// Messages must have been sent within the past 1 week.
#[openapi(tag = "Messaging")]
#[delete("/<target>/messages/bulk", data = "<options>", rank = 1)]
pub async fn bulk_delete_messages(
    db: &State<Database>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    options: Json<v0::OptionsBulkDelete>,
) -> Result<EmptyResponse> {
    let options = options.into_inner();
    options.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    for id in &options.ids {
        if ulid::Ulid::from_string(id)
            .map_err(|_| create_error!(InvalidOperation))?
            .datetime()
            .elapsed()
            .expect("Time went backwards")
            > Duration::from_hours(7 * 24)
        // 7 days
        {
            return Err(create_error!(InvalidOperation));
        }
    }

    let channel = target.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageMessages)?;

    Message::bulk_delete(db, target.id, options.ids.clone()).await?;

    if let Some(server) = channel.server() {
        AuditLogEntryAction::MessageBulkDelete {
            channel: channel.id().to_string(),
            count: options.ids.len(),
        }
        .insert(db, server.to_string(), reason, user.id, None)
        .await;
    };

    Ok(EmptyResponse)
}
