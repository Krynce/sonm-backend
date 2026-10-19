use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{
    AMQP, AuditLogEntryAction, Channel, Database, FieldsMessage, PartialMessage, SystemMessage,
    User, util::reference::Reference,
};
use sonm_models::v0::MessageAuthor;
use sonm_permissions::ChannelPermission;
use sonm_result::{Result, create_error};

use crate::util::audit_log_reason::AuditLogReason;

use crate::util::checks::require_channel_permission;

/// # Pins a message
///
/// Pins a message by its id.
#[openapi(tag = "Messaging")]
#[post("/<target>/messages/<msg>/pin")]
pub async fn message_pin(
    db: &State<Database>,
    amqp: &State<AMQP>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    msg: Reference<'_>,
) -> Result<EmptyResponse> {
    set_pinned(db, amqp, user, reason, target, msg, true).await
}

/// Shared body of [`message_pin`] and `message_unpin`
pub async fn set_pinned(
    db: &Database,
    amqp: &AMQP,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    msg: Reference<'_>,
    pinned: bool,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;

    if !matches!(channel, Channel::DirectMessage { .. }) {
        require_channel_permission(db, &user, &channel, ChannelPermission::ManageMessages).await?;
    }

    let mut message = msg.as_message_in_channel(db, channel.id()).await?;

    match (pinned, message.pinned.unwrap_or_default()) {
        (true, true) => return Err(create_error!(AlreadyPinned)),
        (false, false) => return Err(create_error!(NotPinned)),
        _ => {}
    }

    if pinned {
        let partial = PartialMessage {
            pinned: Some(true),
            ..Default::default()
        };
        message.update(db, partial, vec![]).await?;
    } else {
        message
            .update(db, PartialMessage::default(), vec![FieldsMessage::Pinned])
            .await?;
    }

    let (id, by) = (message.id.clone(), user.id.clone());
    let system_message = if pinned {
        SystemMessage::MessagePinned { id, by }
    } else {
        SystemMessage::MessageUnpinned { id, by }
    };

    system_message
        .into_message(channel.id().to_string())
        .send(
            db,
            Some(amqp),
            MessageAuthor::System {
                username: &user.username,
                avatar: user.avatar.as_ref().map(|file| file.id.as_ref()),
            },
            None,
            None,
            &channel,
            false,
        )
        .await?;

    if let Some(server_id) = channel.server() {
        let (message_id, author, channel_id) = (
            message.id.clone(),
            message.author.clone(),
            message.channel.clone(),
        );
        let action = if pinned {
            AuditLogEntryAction::MessagePin {
                message: message_id,
                author,
                channel: channel_id,
            }
        } else {
            AuditLogEntryAction::MessageUnpin {
                message: message_id,
                author,
                channel: channel_id,
            }
        };

        action
            .insert(
                db,
                server_id.to_string(),
                reason,
                user.id,
                Some(message.author),
            )
            .await;
    }

    Ok(EmptyResponse)
}

#[cfg(test)]
mod test {
    use crate::util::test::PubSubTestHelper;
    use crate::{rocket, util::test::TestHarness};
    use rocket::http::{Header, Status};
    use sonm_database::{
        Member, Message, Server,
        events::client::EventV1,
        util::{idempotency::IdempotencyKey, reference::Reference},
    };
    use sonm_models::v0::{self, SystemMessage};

    #[rocket::async_test]
    async fn pin_message() {
        let harness = TestHarness::new().await;
        let (_, session, user) = harness.new_user().await;

        let (server, channels) = Server::create(
            &harness.db,
            v0::DataCreateServer {
                name: "Test Server".to_string(),
                ..Default::default()
            },
            &user,
            true,
        )
        .await
        .expect("Failed to create test server");

        let (member, channels) = Member::create(&harness.db, &server, &user, Some(channels))
            .await
            .expect("Failed to create member");
        let channel = &channels[0];
        let mut pubsub = PubSubTestHelper::new(channel.id()).await;

        let message = Message::create_from_api(
            &harness.db,
            None,
            channel.clone(),
            v0::DataMessageSend {
                content: Some("Test message".to_string()),
                nonce: None,
                attachments: None,
                replies: None,
                embeds: None,
                masquerade: None,
                interactions: None,
                flags: None,
            },
            v0::MessageAuthor::User(&user.clone().into(&harness.db, Some(&user)).await),
            Some(user.clone().into(&harness.db, Some(&user)).await),
            Some(member.into()),
            user.limits().await,
            IdempotencyKey::unchecked_from_string("0".to_string()),
            false,
            false,
        )
        .await
        .expect("Failed to create message");

        let response = harness
            .client
            .post(format!(
                "/channels/{}/messages/{}/pin",
                channel.id(),
                message.id
            ))
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);
        drop(response);

        pubsub
            .wait_for_event(|event| match event {
                EventV1::Message(message) => match &message.system {
                    Some(SystemMessage::MessagePinned { by, .. }) => {
                        assert_eq!(by, &user.id);

                        true
                    }
                    _ => false,
                },
                _ => false,
            })
            .await;

        pubsub
            .wait_for_event(|event| match event {
                EventV1::MessageUpdate {
                    id,
                    channel: channel_id,
                    data,
                    ..
                } => {
                    assert_eq!(id, &message.id);
                    assert_eq!(channel_id, channel.id());
                    assert_eq!(data.pinned, Some(true));

                    true
                }
                _ => false,
            })
            .await;

        let updated_message = Reference::from_unchecked(&message.id)
            .as_message(&harness.db)
            .await
            .expect("Failed to find updated message");

        assert_eq!(updated_message.pinned, Some(true));
    }
}
