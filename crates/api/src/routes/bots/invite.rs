use rocket::State;
use sonm_database::util::permissions::DatabasePermissionQuery;
use sonm_database::{AMQP, AuditLogEntryAction, Member, PartialMember, PartialRole, Role};
use sonm_database::{Database, User, util::reference::Reference};
use sonm_models::v0;
use sonm_permissions::{
    ChannelPermission, Override, calculate_channel_permissions, calculate_server_permissions,
};
use sonm_result::{Result, create_error};

use rocket::serde::json::Json;
use rocket_empty::EmptyResponse;

use crate::util::audit_log_reason::AuditLogReason;

/// # Invite Bot
///
/// Invite a bot to a server or group by its id.`
#[openapi(tag = "Bots")]
#[post("/<target>/invite?<permissions>", data = "<dest>")]
pub async fn invite_bot(
    db: &State<Database>,
    amqp: &State<AMQP>,
    user: User,
    reason: AuditLogReason,
    target: Reference<'_>,
    permissions: Option<i64>,
    dest: Json<v0::InviteBotDestination>,
) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    let bot = target.as_bot(db).await?;
    if !bot.public && bot.owner != user.id {
        return Err(create_error!(BotIsPrivate));
    }

    let bot_user = db.fetch_user(&bot.id).await?;

    match dest.into_inner() {
        v0::InviteBotDestination::Server { server } => {
            let server = db.fetch_server(&server).await?;

            let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
            let user_permissions = calculate_server_permissions(&mut query).await;
            user_permissions
                .throw_if_lacking_channel_permission(ChannelPermission::ManageServer)?;

            let (mut member, _channels) = Member::create(db, &server, &bot_user, None).await?;

            if let Some(permissions) = permissions {
                user_permissions
                    .throw_if_lacking_channel_permission(ChannelPermission::ManageRole)?;

                let requested = Override {
                    allow: permissions as u64,
                    deny: 0,
                };

                user_permissions
                    .throw_permission_override(Override::default(), &requested)
                    .await?;

                let mut role = Role::create_managed(
                    db,
                    &server,
                    bot_user.username.clone(),
                    bot_user.id.clone(),
                )
                .await?;

                role.update(
                    db,
                    &server.id,
                    PartialRole {
                        permissions: Some(requested.into()),
                        ..Default::default()
                    },
                    vec![],
                )
                .await?;

                AuditLogEntryAction::RoleCreate {
                    role: role.id.clone(),
                    name: role.name.clone(),
                }
                .insert(db, server.id.clone(), reason, user.id.clone(), None)
                .await;

                member
                    .update(
                        db,
                        PartialMember {
                            roles: Some(vec![role.id.clone()]),
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await?;
            }

            Ok(EmptyResponse)
        }
        v0::InviteBotDestination::Group { group } => {
            let mut channel = db.fetch_channel(&group).await?;

            let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
            calculate_channel_permissions(&mut query)
                .await
                .throw_if_lacking_channel_permission(ChannelPermission::InviteOthers)?;

            channel
                .add_user_to_group(db, amqp, &bot_user, &user.id)
                .await
                .map(|_| EmptyResponse)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::util::test::PubSubTestHelper;
    use crate::{rocket, util::test::TestHarness};
    use rocket::http::{ContentType, Header, Status};
    use sonm_database::{Bot, Channel, Server, events::client::EventV1};
    use sonm_models::v0::{self, DataCreateServer};

    #[rocket::async_test]
    async fn invite_bot_to_group() {
        let harness = TestHarness::new().await;
        let (_, session, user) = harness.new_user().await;

        let (bot, _) = Bot::create(&harness.db, TestHarness::rand_string(), &user, None)
            .await
            .expect("`Bot`");

        let group = Channel::create_group(
            &harness.db,
            v0::DataCreateGroup {
                name: TestHarness::rand_string(),
                ..Default::default()
            },
            user.id.to_string(),
        )
        .await
        .unwrap();
        let mut pubsub = PubSubTestHelper::new(group.id()).await;

        let response = harness
            .client
            .post(format!("/bots/{}/invite", bot.id))
            .header(ContentType::JSON)
            .body(
                json!(v0::InviteBotDestination::Group {
                    group: group.id().to_string()
                })
                .to_string(),
            )
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);
        drop(response);

        let event = pubsub
            .wait_for_event(|event| match event {
                EventV1::ChannelGroupJoin { id, .. } => id == group.id(),
                _ => false,
            })
            .await;

        match event {
            EventV1::ChannelGroupJoin { user, .. } => {
                assert_eq!(bot.id, user);
            }
            _ => unreachable!(),
        }
    }

    #[rocket::async_test]
    async fn invite_bot_to_server() {
        let harness = TestHarness::new().await;
        let (_, session, user) = harness.new_user().await;

        let (bot, _) = Bot::create(&harness.db, TestHarness::rand_string(), &user, None)
            .await
            .expect("`Bot`");

        let (server, _) = Server::create(
            &harness.db,
            DataCreateServer {
                name: TestHarness::rand_string(),
                ..Default::default()
            },
            &user,
            false,
        )
        .await
        .unwrap();
        let mut pubsub = PubSubTestHelper::new(&server.id).await;

        let response = harness
            .client
            .post(format!("/bots/{}/invite", bot.id))
            .header(ContentType::JSON)
            .body(
                json!(v0::InviteBotDestination::Server {
                    server: server.id.to_string()
                })
                .to_string(),
            )
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);
        drop(response);

        let event = pubsub
            .wait_for_event(|event| match event {
                EventV1::ServerMemberJoin { id, .. } => id == &server.id,
                _ => false,
            })
            .await;

        match event {
            EventV1::ServerMemberJoin { member, .. } => {
                assert_eq!(bot.id, member.id.user);
            }
            _ => unreachable!(),
        }
    }
}
