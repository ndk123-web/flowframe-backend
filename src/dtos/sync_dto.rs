use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncUserRequest {
    pub email: String,
    pub firebase_uid: String,
    pub type_of_signin: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub id_token: Option<String>,
}
