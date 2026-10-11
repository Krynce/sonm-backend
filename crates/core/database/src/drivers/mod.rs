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
                let mut options = ::mongodb::options::ClientOptions::parse(&uri)
                    .await
                    .map_err(|_| "Failed to parse db connection string.".to_string())?;

                // The driver defaults to a pool of 10 per host, which concurrent requests
                // queue behind, and to a 30 s server selection timeout, which turns a dead
                // MongoDB into 30 s hangs instead of fast errors. Values in the connection
                // string still win: `parse` has already applied them.
                //
                // 20 is deliberately modest: six services connect, so this is a ceiling of
                // 120 connections, against a server that allows ~409 and runs under a
                // 640 MB limit at roughly 1 MB of thread stack per connection. Steady
                // state is ~33 for the whole stack, so this is ample headroom without
                // re-creating the OOM that phase 0 fixed.
                options.max_pool_size = options.max_pool_size.or(Some(20));
                options.min_pool_size = options.min_pool_size.or(Some(5));
                options.connect_timeout = options
                    .connect_timeout
                    .or(Some(std::time::Duration::from_secs(5)));
                options.server_selection_timeout = options
                    .server_selection_timeout
                    .or(Some(std::time::Duration::from_secs(5)));
                // Wire compression is deliberately left off: MongoDB shares the host with
                // the services, so it would only trade CPU for bandwidth we are not short of.

                let client = ::mongodb::Client::with_options(options)
                    .map_err(|_| "Failed to init db connection.".to_string())?;

                Ok(Database::MongoDb(MongoDb(client, database_name)))
            }
            DatabaseInfo::MongoDbFromClient(client, database_name) => {
                Ok(Database::MongoDb(MongoDb(client, database_name)))
            }
        }
    }
}
