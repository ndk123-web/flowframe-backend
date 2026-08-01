use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    pub type_of_signin: Option<String>,
}

