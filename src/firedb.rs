use std::path::PathBuf;
use firestore::{FirestoreDb, FirestoreDbOptions};
use firestore::errors::FirestoreError; 
use crate::models::Firedisplayinfo;

pub struct DbService {
    client: FirestoreDb,
}

impl DbService {
    pub async fn new() -> DbService {
        let options = FirestoreDbOptions::new("dash-demo-7f5f3".to_string());
        let key_path = "/app/firebase.json".into(); 

        let client = FirestoreDb::with_options_service_account_key_file(options, key_path)
            .await
            .unwrap();

        DbService { client }
    }

    pub async fn insert(&self, user: Firedisplayinfo) -> Result<(), FirestoreError> {
        self.client
            .fluent()
            .insert()
            .into("mqtt_db")
            .document_id(&user.id)
            .object(&user)
            .execute::<()>() //Specify return type explicitly
            .await?;

        Ok(())
    }

    pub async fn get_all(&self) -> Result<Vec<Firedisplayinfo>, FirestoreError> {
        let users = self
            .client
            .fluent()
            .select()
            .from("mqtt_db")
            .obj()
            .query()
            .await?;

        Ok(users)
    }

    pub async fn update_by_id(&self, user: Firedisplayinfo) -> Result<(), FirestoreError> {
        self.client
            .fluent()
            .update()
            .in_col("mqtt_db")
            .document_id(&user.id)
            .object(&user)
            .execute::<()>() 
            .await?;

        Ok(())
    }
}
