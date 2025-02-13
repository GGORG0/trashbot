use mongodb::{error::Result, Client, Database};
use std::sync::Arc;
use tokio::sync::OnceCell;

pub struct MongoConnectionProvider {
    client: Client,
    db: Database,
}

impl MongoConnectionProvider {
    // Create a new MongoConnectionProvider
    pub async fn new(uri: &str, db_name: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(db_name);
        Ok(Self { client, db })
    }

    pub fn database(&self) -> &Database {
        &self.db
    }
}

// Global provider instance accessible from anywhere
static GLOBAL_PROVIDER: OnceCell<Arc<MongoConnectionProvider>> = OnceCell::const_new();

// Initialize the global connection provider
pub async fn init(uri: &str, db_name: &str) -> Result<()> {
    let provider = MongoConnectionProvider::new(uri, db_name).await?;
    GLOBAL_PROVIDER.set(Arc::new(provider)).map_err(|_| {
        mongodb::error::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Provider already initialized",
        ))
    })?;
    Ok(())
}

// Get the database instance from the global provider.
pub fn get_db() -> &'static Database {
    // Assumes that init() was called beforehand.
    &GLOBAL_PROVIDER
        .get()
        .expect("MongoConnectionProvider not initialized")
        .database()
}

/* Usage Guide:

1. In your main file, initialize the connection:
   #[tokio::main]
   async fn main() {
       let uri = "mongodb://localhost:27017";
       let db_name = "my_database";
       mongo_connection_provider::init(uri, db_name)
           .await
           .expect("Failed to initialize MongoDB connection");

       // Now you can use the database anywhere:
       let db = mongo_connection_provider::get_db();
       // Perform operations using `db`.
   }

2. In other modules, import and use as follows:
   use mongo_connection_provider::get_db;
*/
