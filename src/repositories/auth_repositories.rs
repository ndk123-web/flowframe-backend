use crate::models::user_model::User;
use anyhow::Result;
use bson::doc;
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

    pub async fn create_user(&self, mut user: User) -> Result<User> {
        let insert_result = self.user_collection.insert_one(&user).await?;
        if let Some(inserted_id) = insert_result.inserted_id.as_object_id() {
            user.id = Some(inserted_id);
        }
        Ok(user)
    }
}
