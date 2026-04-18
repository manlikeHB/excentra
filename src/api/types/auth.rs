use validator::Validate;

#[derive(serde::Deserialize, Validate, utoipa::ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password should be at least 8 characters"))]
    pub password: String,
}

#[derive(serde::Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password should be at least 8 characters"))]
    pub password: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
}
