use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};

pub async fn init() {
    //console_subscriber::init();
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // List the names of the databases in that deployment.
    for db_name in client.list_database_names(None, None).await {
        println!("{:?}", db_name);
    }
    let db = client.database("PaulCoin");

    // List the names of the collections in that database.
    for collection_name in db.list_collection_names(None).await {
        println!("{:?}", collection_name);
    }
    // Get a handle to a collection in the database.
    let collection = db.collection::<Document>("books");

    let docs = vec![
        doc! { "title": "1984", "author": "George Orwell" },
        doc! { "title": "Animal Farm", "author": "George Orwell" },
        doc! { "title": "The Great Gatsby", "author": "F. Scott Fitzgerald" },
    ];

    // Insert some documents into the "mydb.books" collection.
    collection.insert_many(docs, None).await.unwrap();
}