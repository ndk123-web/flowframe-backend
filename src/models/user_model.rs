#[allow(dead_code)]
#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub type_of_signin: String,
}
