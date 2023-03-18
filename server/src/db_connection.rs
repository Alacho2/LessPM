use mongodb::{Client, Collection, Database};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
// use mongodb::bson::Bson;
use mongodb::options::ClientOptions;
use std::fmt::Display;
use serde::{Deserialize, Serialize};

pub struct DbConnection {
  db: Database
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultEntry {
  // _id: typed automatically by mongo driver.
  pub username: String,
  pub password: String,
  pub website: String,
  pub nonce: [u8; 12],
  pub random_padding: Vec<u8>,
}

impl DbConnection {
  pub async fn new() -> Self {
    let url = "mongodb://localhost:27017";
    let mut client_options = ClientOptions::parse(url).await.unwrap();
    client_options.app_name = Some("LessPM".to_string());
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("lesspm");
    Self {
      db,
    }
  }

  // Takes a processed document and inserts it into the db
  pub async fn insert_one(&self, collection_name: &str, vault_entry: VaultEntry) {
    let collection = &self.db.collection(collection_name);

    println!("Got here");

    match collection.insert_one(vault_entry, None).await {
      Ok(doc) => {
        println!("Record got inserted with ID: {}", doc.inserted_id);
        let something: Option<VaultEntry> = collection.find_one(Some(doc! {
          "_id": doc.inserted_id.clone()
        }), None).await.expect("Document not found");
        dbg!("Returned values: {}", something.unwrap());
        println!("Record got inserted with ID: {}", doc.inserted_id)
      },
      Err(e) => {
        println!("Didn't manage to insert it: {}", e);
      }
    }
  }

  pub async fn get_one(
    &self,
    collection_name: &str,
    id: ObjectId
  ) -> Option<VaultEntry> {
    let collection: &Collection<VaultEntry>
      = &self.db.collection(collection_name);

    match collection.find_one(Some(doc! {
      "_id": id,
    }), None).await {
        Ok(vault_entry) => {
          return vault_entry;
        },
        Err(_) => None,
    }
  }
}
