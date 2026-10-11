use futures::future::try_join_all;
use sonm_database::{
    Database, Server,
    voice::{VoiceClient, sync_voice_permissions},
};
use sonm_result::Result;

/// Re-sync LiveKit permissions for every channel of a server
///
/// Five routes used to walk `server.channels` fetching one channel at a time and waiting
/// for each LiveKit round-trip before starting the next: one batched query, then all the
/// syncs in flight at once.
pub async fn sync_server_voice_permissions(
    db: &Database,
    voice_client: &VoiceClient,
    server: &Server,
    role_id: Option<&str>,
) -> Result<()> {
    let channels = db.fetch_channels(&server.channels).await?;

    try_join_all(
        channels.iter().map(|channel| {
            sync_voice_permissions(db, voice_client, channel, Some(server), role_id)
        }),
    )
    .await?;

    Ok(())
}
