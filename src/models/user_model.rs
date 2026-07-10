#[allow(dead_code)]
#[derive(Debug)]
pub struct User {
    id: i64,
    email: String,
    password_hash: String,
    type_of_signin: String,
}
