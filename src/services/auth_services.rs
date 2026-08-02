use crate::dtos::auth_response_dto::{AuthResponse, UserData};
use crate::dtos::signin_dto::SignInRequest;
use crate::dtos::signup_dto::SignUpRequest;
use crate::dtos::sync_dto::SyncUserRequest;
use crate::models::user_model::User;
use crate::repositories::auth_repositories::AuthRepository;
use crate::utils::firebase_auth::verify_firebase_id_token;
use crate::utils::jwt::generate_jwt;
use crate::utils::password_hash::{hash_password, verify_password};
use anyhow::{anyhow, Result};
use bson::DateTime;

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
        let now = DateTime::now();

        let new_user = User {
            id: None,
            firebase_uid: None,
            email: req.email.clone(),
            name: None,
            avatar: None,
            password_hash: Some(password_hash),
            type_of_signin: type_of_signin.clone(),
            created_at: now,
            updated_at: now,
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
                firebase_uid: created_user.firebase_uid,
                name: created_user.name,
                avatar: created_user.avatar,
            },
        })
    }

    pub async fn signin(&self, req: SignInRequest) -> Result<AuthResponse> {
        let user = self
            .repo
            .find_by_email(&req.email)
            .await?
            .ok_or_else(|| anyhow!("Invalid email or password"))?;

        let hash_str = user
            .password_hash
            .as_deref()
            .ok_or_else(|| anyhow!("Invalid authentication type for password login"))?;

        let is_valid = verify_password(&req.password, hash_str)?;
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
                firebase_uid: user.firebase_uid,
                name: user.name,
                avatar: user.avatar,
            },
        })
    }

    pub async fn sync_user(&self, req: SyncUserRequest) -> Result<AuthResponse> {
        let type_of_signin = req.type_of_signin.unwrap_or_else(|| "firebase".to_string());

        // Verify Firebase ID token if provided
        if let Some(ref id_token) = req.id_token {
            if !id_token.is_empty() {
                let claims = verify_firebase_id_token(id_token).await?;
                // Ensure the token's UID matches the requested firebase_uid
                if claims.sub != req.firebase_uid {
                    return Err(anyhow!("Firebase ID Token UID mismatch"));
                }
            }
        }

        let db_user = self
            .repo
            .upsert_firebase_user(
                &req.email,
                &req.firebase_uid,
                &type_of_signin,
                req.name,
                req.avatar,
            )
            .await?;

        let user_id_str = db_user.id.map(|id| id.to_hex()).unwrap_or_default();

        let token = generate_jwt(&user_id_str, &db_user.email, &self.jwt_secret)?;

        Ok(AuthResponse {
            access_token: token,
            user: UserData {
                id: user_id_str,
                email: db_user.email,
                type_of_signin: db_user.type_of_signin,
                firebase_uid: db_user.firebase_uid,
                name: db_user.name,
                avatar: db_user.avatar,
            },
        })
    }
}
