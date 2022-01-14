use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};

pub async fn init() {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("myclient".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("myclient");

    // List the names of the collections in that database.
    for collection_name in db.list_collection_names(None).await {
        println!("{:?}", collection_name);
    }
    // Get a handle to a collection in the database.
    let collection = db.collection::<Document>("block");

    let docs = vec![
        doc! { 
            "transaction": "123",
            "author": "Paul Cibier",
            "Receiver" : "Remyx",
            "previous_block": "000000000000000000000000000000000",
            "hash_block" : "287Y87Y8Y8Y88T7TT8TUUGYGGHGHGHJGJ"
        },
    ];

    // Insert some documents into the "mydb.books" collection.
    collection.insert_many(docs, None).await.unwrap();
}