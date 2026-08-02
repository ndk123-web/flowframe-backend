use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const FIREBASE_PROJECT_ID: &str = "flowframe-2168c";
const GOOGLE_PUBLIC_KEYS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct FirebaseClaims {
    pub iss: String,
    pub aud: String,
    pub auth_time: u64,
    pub user_id: String,
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

pub async fn verify_firebase_id_token(id_token: &str) -> Result<FirebaseClaims> {
    // 1. Decode token header to get kid (key ID)
    let header = decode_header(id_token)?;
    let kid = header
        .kid
        .ok_or_else(|| anyhow!("Firebase ID token header missing 'kid'"))?;

    // 2. Fetch Google public x509 certificates
    let client = reqwest::Client::new();
    let res = client.get(GOOGLE_PUBLIC_KEYS_URL).send().await?;
    if !res.status().is_success() {
        return Err(anyhow!("Failed to fetch Google public keys for Firebase"));
    }

    let keys: HashMap<String, String> = res.json().await?;
    let cert_pem = keys
        .get(&kid)
        .ok_or_else(|| anyhow!("Google public key not found for kid: {}", kid))?;

    // 3. Create decoding key from x509 certificate PEM
    let decoding_key = DecodingKey::from_rsa_pem(cert_pem.as_bytes())?;

    // 4. Validate claims
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[FIREBASE_PROJECT_ID]);
    validation.set_issuer(&[&format!("https://securetoken.google.com/{}", FIREBASE_PROJECT_ID)]);

    let token_data = decode::<FirebaseClaims>(id_token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}
