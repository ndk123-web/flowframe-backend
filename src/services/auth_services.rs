use crate::dtos::auth_response_dto::{AuthResponse, UserData};
use crate::dtos::signin_dto::SignInRequest;
use crate::dtos::signup_dto::SignUpRequest;
use crate::models::user_model::User;
use crate::repositories::auth_repositories::AuthRepository;
use crate::utils::jwt::generate_jwt;
use crate::utils::password_hash::{hash_password, verify_password};
use anyhow::{anyhow, Result};

#[derive(Clone)]
pub struct AuthService {
    repo: AuthRepository,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(repo: AuthRepository, jwt_secret: String) -> Self {
        Self { repo, jwt_secret }
    }

    pub async fn signup(&self, req: SignUpRequest) -> Result<AuthResponse> {
        if self.repo.find_by_email(&req.email).await?.is_some() {
            return Err(anyhow!("User with this email already exists"));
        }

        let password_hash = hash_password(&req.password)?;
        let type_of_signin = req.type_of_signin.unwrap_or_else(|| "email".to_string());

        let new_user = User {
            id: None,
            email: req.email.clone(),
            password_hash,
            type_of_signin: type_of_signin.clone(),
        };

        let created_user = self.repo.create_user(new_user).await?;
        let user_id_str = created_user
            .id
            .map(|id| id.to_hex())
            .unwrap_or_default();

        let token = generate_jwt(&user_id_str, &created_user.email, &self.jwt_secret)?;

        Ok(AuthResponse {
            access_token: token,
            user: UserData {
                id: user_id_str,
                email: created_user.email,
                type_of_signin: created_user.type_of_signin,
            },
        })
    }

    pub async fn signin(&self, req: SignInRequest) -> Result<AuthResponse> {
        let user = self
            .repo
            .find_by_email(&req.email)
            .await?
            .ok_or_else(|| anyhow!("Invalid email or password"))?;

        let is_valid = verify_password(&req.password, &user.password_hash)?;
        if !is_valid {
            return Err(anyhow!("Invalid email or password"));
        }

        let user_id_str = user.id.map(|id| id.to_hex()).unwrap_or_default();

        let token = generate_jwt(&user_id_str, &user.email, &self.jwt_secret)?;

        Ok(AuthResponse {
            access_token: token,
            user: UserData {
                id: user_id_str,
                email: user.email,
                type_of_signin: user.type_of_signin,
            },
        })
    }
}
