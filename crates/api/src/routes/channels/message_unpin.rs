use rocket::State;
use rocket_empty::EmptyResponse;
use sonm_database::{AMQP, Database, User, util::reference::Reference};
use sonm_result::Result;

use crate::util::audit_log_reason::AuditLogReason;

use super::message_pin::set_pinned;

/// # Unpins a message
///
/// Unpins a message by its id.
#[openapi(tag = "Messaging")]
#[delete("/<target>/messages/<msg>/pin")]
pub async fn message_unpin(
    db: &State<Database>,
    amqp: &State<AMQP>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    msg: Reference<'_>,
) -> Result<EmptyResponse> {
    set_pinned(db, amqp, user, reason, target, msg, false).await
}

#[cfg(test)]
mod test {
    use crate::util::test::PubSubTestHelper;
    use crate::{rocket, util::test::TestHarness};
    use rocket::http::{Header, Status};
    use sonm_database::{
        Member, Message, PartialMessage, Server,
        events::client::EventV1,
        util::{idempotency::IdempotencyKey, reference::Reference},
    };
    use sonm_models::v0::{self, FieldsMessage, SystemMessage};

    #[rocket::async_test]
    async fn unpin_message() {
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

        let channel = &channels[0];
        let mut pubsub = PubSubTestHelper::new(channel.id()).await;

        Member::create(&harness.db, &server, &user, Some(channels.clone()))
            .await
            .expect("Failed to create member");
        let member = Reference::from_unchecked(&user.id)
            .as_member(&harness.db, &server.id)
            .await
            .expect("Failed to get member");

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

        harness
            .db
            .update_message(
                &message.id,
                &PartialMessage {
                    pinned: Some(true),
                    ..Default::default()
                },
                vec![],
            )
            .await
            .expect("Failed to update message");

        let response = harness
            .client
            .delete(format!(
                "/channels/{}/messages/{}/pin",
                channel.id(),
                &message.id
            ))
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);
        drop(response);

        pubsub
            .wait_for_event(|event| match event {
                EventV1::Message(message) => match &message.system {
                    Some(SystemMessage::MessageUnpinned { by, .. }) => {
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
                EventV1::MessageUpdate { id, clear, .. } => {
                    assert_eq!(&message.id, id);
                    assert_eq!(clear, &[FieldsMessage::Pinned]);

                    true
                }
                _ => false,
            })
            .await;

        let updated_message = Reference::from_unchecked(&message.id)
            .as_message(&harness.db)
            .await
            .expect("Failed to find updated message");

        assert_eq!(updated_message.pinned, None);
    }
}
