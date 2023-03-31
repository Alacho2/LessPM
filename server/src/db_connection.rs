use anyhow::anyhow;
use axum::http::StatusCode;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{doc};
use mongodb::bson::oid::ObjectId;
use mongodb::options::{ClientOptions, FindOptions};
use mongodb::error::Result as MongoDbResult;
use regex::Regex;
use tokio_stream::StreamExt;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{Passkey, Uuid};

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
  pub uuid: String,
  pub key_padding: Vec<u8>,
  pub key_salt: [u8; 16],
  pub argon_salt: [u8; 16],
}

#[derive(Serialize, Deserialize)]
pub struct VaultEntryStripped {
  _id: ObjectId,
  pub username: String,
  pub website: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredUser {
  pub username: String,
  pub uuid: Uuid,
  pub passkey: Passkey,
}

pub fn is_valid_username(username: &str) -> bool {
  let re = Regex::new(r"^[a-zA-Z0-9]{3,24}$").unwrap();
  re.is_match(username)
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
  pub async fn insert_one_to_vault(
    &self,
    vault_entry: VaultEntry
  ) -> Result<StatusCode, StatusCode> {
    let collection: &Collection<VaultEntry> = &self.db.collection("vault");

    match collection.insert_one(vault_entry, None).await {
      Ok(_) => {
        Ok(StatusCode::CREATED)
      },
      Err(e) => {
        println!("Didn't manage to insert it: {}", e);
        Err(StatusCode::BAD_REQUEST)
      }
    }
  }

  pub async fn get_one_from_vault(
    &self,
    id: ObjectId
  ) -> Option<VaultEntry> {
    let collection: &Collection<VaultEntry>
      = &self.db.collection("vault");

    match collection.find_one(Some(doc! {
      "_id": id,
    }), None).await {
        Ok(vault_entry) => vault_entry,
        Err(_) => None,
    }
  }

  pub async fn get_passwords(
    &self,
    collection_name: &str,
    uuid: &String
  ) -> anyhow::Result<Vec<VaultEntryStripped>> {
    let collection: &Collection<VaultEntryStripped> = &self.db.collection(collection_name);

    let find_options = FindOptions::builder()
        .projection(doc! { "_id": 1, "password": 0, "nonce": 0, "random_padding": 0 })
        .build();

    let cursor = collection.find(Some(doc! {
      "uuid": uuid,
    }), find_options).await?;

    let v: Vec<MongoDbResult<VaultEntryStripped>> = cursor.collect().await;
    let entries: Result<Vec<VaultEntryStripped>, _> = v.into_iter().collect();
    entries.map_err(|e| anyhow!("Failed to collect entries: {}", e))
  }

  pub async fn get_registered_user(
    &self,
    username: String
  ) -> Option<RegisteredUser> {
    let collection: &Collection<RegisteredUser> =
      &self.db.collection("users");

    if !is_valid_username(&username) {
      return None;
    }

    match collection.find_one(Some(doc! {
      "username": username,
    }), None).await {
      Ok(user) => user,
      Err(_) => None,
    }
  }

  pub async fn register_user(
    &self,
    user: RegisteredUser,
  ) {
    let collection: &Collection<RegisteredUser> =
      &self.db.collection("users");

    if !is_valid_username(&user.username) {
      // Handle this somehow.
    }

    match collection.insert_one(user, None).await {
      Ok(_) => {
        println!("Record inserted")
      },
      Err(e) => {
        println!("Didn't manage to insert it: {}", e)
      }
    }
  }
}
