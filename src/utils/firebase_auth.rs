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

/*
    we have idToken of firebase
    we need to check that whether the signature inside id_token is created by google or not 

    1. Read JWT Header
    ↓
    2. Extract kid
    ↓
    3. Download Google's certificates
    ↓
    4. Find certificate matching kid
    ↓
    5. Extract Google Public Key
    ↓
    6. Verify Signature
    ↓
    7. Verify exp
    ↓
    8. Verify aud
    ↓
    9. Verify iss
    ↓
    10. Return Claims
*/
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
    // here decoding_key is parsed public key from cert_pem
    let decoding_key = DecodingKey::from_rsa_pem(cert_pem.as_bytes())?;

    // 4. Validate claims
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[FIREBASE_PROJECT_ID]);
    validation.set_issuer(&[&format!("https://securetoken.google.com/{}", FIREBASE_PROJECT_ID)]);

    let token_data = decode::<FirebaseClaims>(id_token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}
