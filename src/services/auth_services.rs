use crate::dtos::signup_dto::SignUpRequest;
use crate::models::user_model::User;
use crate::repositories::auth_repositories::signup_repo;
use sqlx::SqlitePool;

#[allow(dead_code)]
pub async fn signup_service(pool: &SqlitePool, data: SignUpRequest) {
    let email = data.email;
    let password_hash = data.password;
    let type_of_signin = data.type_of_signin;

    let user = User {
        id: 1,
        email,
        password_hash,
        type_of_signin,
    };

    signup_repo(pool, user);
}
