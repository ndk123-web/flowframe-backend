use sqlx::SqlitePool;

use crate::models::user_model::User;

#[allow(dead_code)]
pub async fn signup_repo(pool: &SqlitePool, data: User) -> anyhow::Result<User> {
    let user = User {
        id: 1,
        email: String::from("ndk@gmail.com"),
        password_hash: String::from("pass"),
        type_of_signin: String::from("google"),
    };

    println!("{:#?}", pool);
    println!("{:#?}", data);

    return anyhow::Ok(user);
}
