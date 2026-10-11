use futures::{future::try_join_all, try_join};
use rocket::State;
use sonm_database::{
    Database, RemovalIntention, User,
    util::reference::Reference,
    voice::{
        UserVoiceChannel, VoiceClient, delete_voice_channel, get_user_voice_channel_in_server,
        remove_user_from_voice_channel,
    },
};
use sonm_models::v0;
use sonm_result::Result;

use rocket_empty::EmptyResponse;

/// # Delete / Leave Server
///
/// Deletes a server if owner otherwise leaves.
#[openapi(tag = "Server Information")]
#[delete("/<target>?<options..>")]
pub async fn delete(
    db: &State<Database>,
    voice_client: &State<VoiceClient>,
    user: User,
    target: Reference<'_>,
    options: v0::OptionsServerDelete,
) -> Result<EmptyResponse> {
    let (server, member) = try_join!(target.as_server(db), db.fetch_member(target.id, &user.id))?;

    if server.owner == user.id {
        // One LiveKit round-trip per channel, all in flight at once.
        try_join_all(server.channels.iter().map(|channel_id| {
            let voice_channel = UserVoiceChannel {
                id: channel_id.clone(),
                server_id: Some(server.id.clone()),
            };

            async move { delete_voice_channel(voice_client, &voice_channel).await }
        }))
        .await?;

        server.delete(db).await
    } else {
        if let Some(channel_id) = get_user_voice_channel_in_server(&user.id, &server.id).await? {
            remove_user_from_voice_channel(
                voice_client,
                &UserVoiceChannel {
                    id: channel_id,
                    server_id: Some(server.id.clone()),
                },
                &user.id,
            )
            .await?;
        };

        member
            .remove(
                db,
                &server,
                RemovalIntention::Leave,
                options.leave_silently.unwrap_or_default(),
            )
            .await
    }
    .map(|_| EmptyResponse)
}
