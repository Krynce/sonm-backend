use redis_kiss::{AsyncCommands, Conn};

/// Get set members as string
pub async fn __get_set_members_as_string(conn: &mut Conn, key: &str) -> Vec<String> {
    conn.smembers::<_, Vec<String>>(key)
        .await
        .expect("could not get set members as string")
}

/// Delete key by id
pub async fn __delete_key(conn: &mut Conn, id: &str) {
    conn.del::<_, ()>(id)
        .await
        .expect("could not delete key by id");
}
