use ::mongodb::{ClientSession, SessionCursor};
use serde::Deserialize;
use sonm_result::{Result, ToSonmError};

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ChunkedDatabaseGenerator<T> {
    MongoDb {
        session: ClientSession,
        cursor: SessionCursor<T>,
    },
}

impl<T: for<'d> Deserialize<'d> + Clone> ChunkedDatabaseGenerator<T> {
    pub fn new_mongo(session: ClientSession, cursor: SessionCursor<T>) -> Self {
        Self::MongoDb { session, cursor }
    }

    pub async fn next(&mut self) -> Result<Option<T>> {
        match self {
            Self::MongoDb { session, cursor } => {
                cursor.next(session).await.transpose().to_internal_error()
            }
        }
    }

    pub async fn next_n(&mut self, n: usize) -> Result<Option<Vec<T>>> {
        let mut docs = Vec::new();

        while docs.len() < n {
            if let Some(doc) = self.next().await? {
                docs.push(doc);
            } else if docs.is_empty() {
                return Ok(None);
            } else {
                break;
            }
        }

        Ok(Some(docs))
    }
}
