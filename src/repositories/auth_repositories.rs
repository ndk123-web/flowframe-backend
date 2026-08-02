use crate::models::user_model::User;
use anyhow::Result;
use bson::{doc, DateTime};
use mongodb::{Collection, Database};

#[derive(Clone)]
pub struct AuthRepository {
    user_collection: Collection<User>,
}

impl AuthRepository {
    pub fn new(db: &Database) -> Self {
        let user_collection = db.collection::<User>("users");
        Self { user_collection }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let filter = doc! { "email": email };
        let user = self.user_collection.find_one(filter).await?;
        Ok(user)
    }

    pub async fn find_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<User>> {
        let filter = doc! { "firebase_uid": firebase_uid };
        let user = self.user_collection.find_one(filter).await?;
        Ok(user)
    }

    pub async fn create_user(&self, mut user: User) -> Result<User> {
        let insert_result = self.user_collection.insert_one(&user).await?;
        if let Some(inserted_id) = insert_result.inserted_id.as_object_id() {
            user.id = Some(inserted_id);
        }
        Ok(user)
    }

    pub async fn upsert_firebase_user(
        &self,
        email: &str,
        firebase_uid: &str,
        type_of_signin: &str,
        name: Option<String>,
        avatar: Option<String>,
    ) -> Result<User> {
        if let Some(existing_user) = self.find_by_email(email).await? {
            // Update existing user with firebase_uid and latest details
            let filter = doc! { "email": email };
            let now = DateTime::now();
            let mut update_doc = doc! {
                "firebase_uid": firebase_uid,
                "type_of_signin": type_of_signin,
                "updated_at": now,
            };

            if let Some(n) = name {
                update_doc.insert("name", n);
            }
            if let Some(a) = avatar {
                update_doc.insert("avatar", a);
            }

            self.user_collection
                .update_one(filter, doc! { "$set": update_doc })
                .await?;

            let updated_user = self.find_by_email(email).await?.unwrap();
            Ok(updated_user)
        } else {
            // Create new user
            let now = DateTime::now();
            let new_user = User {
                id: None,
                firebase_uid: Some(firebase_uid.to_string()),
                email: email.to_string(),
                name,
                avatar,
                password_hash: None,
                type_of_signin: type_of_signin.to_string(),
                created_at: now,
                updated_at: now,
            };
            self.create_user(new_user).await
        }
    }
}
