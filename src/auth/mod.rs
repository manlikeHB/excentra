use crate::db::models::user::UserRole;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error,
};
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    user_id: Uuid,
    role: UserRole,
    exp: i64,
}

impl Claims {
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn role(&self) -> UserRole {
        self.role
    }
    pub fn exp(&self) -> i64 {
        self.exp
    }
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}

pub fn create_token(user_id: Uuid, role: UserRole, secret: &str) -> Result<String, Error> {
    let claims = Claims {
        user_id,
        role,
        //TODO: reduce exp when in production
        exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, Error> {
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

        let hash = hash_password(&password).unwrap();

        assert!(verify_password(&password, &hash).unwrap());
    }

    #[test]
    fn test_jwt_round_trip() {
        let jwt_secret = "test_secret";
        let user_id = Uuid::new_v4();
        let token = create_token(user_id, UserRole::User, jwt_secret).unwrap();
        let claims = verify_token(&token, jwt_secret).unwrap();
        assert_eq!(claims.user_id, user_id);
    }
}
