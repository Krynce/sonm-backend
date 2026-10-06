mod mongodb;

use rand::Rng;
use sonm_config::config;

pub use self::mongodb::*;

/// Database information to use to create a client
pub enum DatabaseInfo {
    /// Connect using the configured MongoDB, or an empty test database when `TEST_DB` is set
    Auto,
    /// Create an empty testing database with the given name
    Test(String),
    /// Connect to MongoDB
    MongoDb { uri: String, database_name: String },
    /// Use existing MongoDB connection
    MongoDbFromClient(::mongodb::Client, String),
}

/// Database
#[derive(Clone, Debug)]
pub enum Database {
    /// MongoDB database
    MongoDb(MongoDb),
}

impl DatabaseInfo {
    /// Create a database client from the given database information
    #[async_recursion]
    pub async fn connect(self) -> Result<Database, String> {
        let config = config().await;

        match self {
            DatabaseInfo::Auto => {
                if std::env::var("TEST_DB").is_ok() {
                    DatabaseInfo::Test(format!(
                        "sonm_test_{}",
                        rand::thread_rng().gen_range(1_000_000..10_000_000)
                    ))
                    .connect()
                    .await
                } else {
                    DatabaseInfo::MongoDb {
                        uri: config.database.mongodb,
                        database_name: config.database.name,
                    }
                    .connect()
                    .await
                }
            }
            DatabaseInfo::Test(database_name) => {
                DatabaseInfo::MongoDb {
                    uri: config.database.mongodb,
                    database_name,
                }
                .connect()
                .await
            }
            DatabaseInfo::MongoDb { uri, database_name } => {
                let client = ::mongodb::Client::with_uri_str(uri)
                    .await
                    .map_err(|_| "Failed to init db connection.".to_string())?;

                Ok(Database::MongoDb(MongoDb(client, database_name)))
            }
            DatabaseInfo::MongoDbFromClient(client, database_name) => {
                Ok(Database::MongoDb(MongoDb(client, database_name)))
            }
        }
    }
}
