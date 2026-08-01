use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}
