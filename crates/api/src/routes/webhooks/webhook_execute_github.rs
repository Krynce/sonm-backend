use revolt_rocket_okapi::{
    r#gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
    revolt_okapi::openapi3::{MediaType, Parameter, ParameterValue},
};
use rocket::{Request, State, http::Status, request::FromRequest};
use schemars::schema::SchemaObject;
use sonm_database::{AMQP, Database, Message, util::reference::Reference};
use sonm_models::v0::{MessageAuthor, Webhook};
use sonm_result::{Error, Result, create_error};
use ulid::Ulid;

use super::github::{format::event_embed, types::convert_event};
use crate::util::checks::validate;

#[derive(Debug, JsonSchema)]
pub struct EventHeader<'r>(pub &'r str);

impl std::ops::Deref for EventHeader<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for EventHeader<'r> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let headers = request.headers();
        let Some(event) = headers.get_one("X-GitHub-Event") else {
            return rocket::request::Outcome::Error((
                Status::BadRequest,
                create_error!(InvalidOperation),
            ));
        };

        rocket::request::Outcome::Success(Self(event))
    }
}

impl<'r> OpenApiFromRequest<'r> for EventHeader<'r> {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> revolt_rocket_okapi::Result<RequestHeaderInput> {
        let mut content = schemars::Map::new();
        content.insert(
            "X-Github-Event".to_string(),
            MediaType {
                schema: Some(SchemaObject {
                    string: Some(Box::default()),
                    ..Default::default()
                }),
                example: None,
                examples: None,
                encoding: schemars::Map::new(),
                extensions: schemars::Map::new(),
            },
        );

        Ok(RequestHeaderInput::Parameter(Parameter {
            name: "X-Github-Event".to_string(),
            location: "header".to_string(),
            required: true,
            description: Some("The name of the github event".to_string()),
            deprecated: false,
            allow_empty_value: false,
            value: ParameterValue::Content { content },
            extensions: schemars::Map::new(),
        }))
    }
}

/// # Executes a webhook specific to github
///
/// Executes a webhook specific to github and sends a message containing the relevant info about the event
#[openapi(tag = "Webhooks")]
#[post("/<webhook_id>/<token>/github", data = "<data>")]
pub async fn webhook_execute_github(
    db: &State<Database>,
    amqp: &State<AMQP>,
    webhook_id: Reference<'_>,
    token: &str,
    event: EventHeader<'_>,
    data: String,
) -> Result<()> {
    let webhook = webhook_id.as_webhook(db).await?;
    webhook.assert_token(token)?;

    let channel = db.fetch_channel(&webhook.channel_id).await?;
    let event = convert_event(&data, &event)?;

    let Some(sendable_embed) = event_embed(event) else {
        return Ok(());
    };

    validate(&sendable_embed)?;

    let message_id = Ulid::new().to_string();

    let mut message = Message {
        id: message_id,
        author: webhook.id.clone(),
        channel: webhook.channel_id.clone(),
        webhook: Some(std::convert::Into::<Webhook>::into(webhook.clone()).into()),
        ..Default::default()
    };

    #[allow(clippy::disallowed_methods)]
    message.attach_sendable_embed(db, sendable_embed).await?;
    message
        .send(
            db,
            Some(amqp),
            MessageAuthor::Webhook(&webhook.into()),
            None,
            None,
            &channel,
            false,
        )
        .await
}
