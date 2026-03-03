use chrono::Utc;
use dotenvy::dotenv;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error,
};
use std::ops::Add;
use uuid::Uuid;

use crate::db::models::user::UserRole;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    user_id: Uuid,
    role: UserRole,
    exp: chrono::DateTime<Utc>,
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, Default::default())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}

pub fn create_token(user_id: Uuid, role: UserRole) -> Result<String, Error> {
    dotenv().ok();

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let claims = Claims {
        user_id,
        role,
        exp: chrono::Utc::now().add(chrono::Duration::minutes(10)),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, Error> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let claims = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?
    .claims;
    Ok(claims)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_password_and_hash_match() {
        let password = String::from("Hello");

        if let Ok(h) = hash_password(&password) {
            assert_eq!(verify_password(&password, &h).unwrap(), true);
        }
    }
}
