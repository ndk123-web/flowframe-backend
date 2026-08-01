use mongodb::{Client, Database};

pub async fn create_database(uri: &str, database_name: &str) -> Database {
    let client = Client::with_uri_str(uri)
        .await
        .expect("MongoDB Connection Failed");

    println!("All sett");
    client.database(database_name)
}
