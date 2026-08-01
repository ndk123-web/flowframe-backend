use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> Result<String> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

pub fn verify_password(password: &str, hash_str: &str) -> Result<bool> {
    let valid = verify(password, hash_str)?;
    Ok(valid)
}
