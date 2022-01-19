use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{doc, Document};
use ring::{
    rand,
    signature::{self, KeyPair},
};

pub async fn init() {
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
    const MESSAGE: &[u8] = b"hello, world";
    let sig = key_pair.sign(MESSAGE);

    let peer_public_key_bytes = key_pair.public_key().as_ref();
    let peer_public_key =
    signature::UnparsedPublicKey::new(&signature::ED25519, peer_public_key_bytes);
    peer_public_key.verify(MESSAGE, sig.as_ref()).unwrap();

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