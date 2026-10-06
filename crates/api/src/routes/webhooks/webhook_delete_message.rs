use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{Database, util::reference::Reference};
use sonm_result::{Result, create_error};

/// # Deletes a webhook message
///
/// Deletes a message sent by a webhook
#[openapi(tag = "Webhooks")]
#[delete("/<webhook_id>/<token>/<message_id>")]
pub async fn webhook_delete_message(
    db: &State<Database>,
    webhook_id: Reference<'_>,
    token: String,
    message_id: Reference<'_>,
) -> Result<EmptyResponse> {
    let webhook = webhook_id.as_webhook(db).await?;
    webhook.assert_token(&token)?;

    let message = message_id.as_message(db).await?;

    if message.author != webhook.id {
        return Err(create_error!(CannotDeleteMessage));
    }

    message.delete(db).await.map(|_| EmptyResponse)
}
