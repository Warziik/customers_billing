use argon2::{Argon2, PasswordHash, PasswordVerifier};
use hmac::Hmac;
use jwt::{SignWithKey, VerifyWithKey};
use poem::error::InternalServerError;
use poem::web::{Data};
use poem::{Request, Result};
use poem_openapi::{ApiResponse, Object, OpenApi, SecurityScheme, Tags};
use poem_openapi::payload::Json;
use poem_openapi::__private::serde;
use poem_openapi::auth::{Bearer};
use poem_openapi::payload::PlainText;
use sha2::Sha256;
use sqlx::{Pool, Postgres};
use crate::user::User;
use serde::Deserialize;

pub const SERVER_KEY: &[u8] = b"123456";
pub type ServerKey = Hmac<Sha256>;

pub struct AuthApi;

#[derive(Tags)]
enum ApiTags {
    Auth
}

#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    key_name = "Authorization",
    key_in = "header",
    checker = "api_checker"
)]
pub struct AuthProvider(pub User);

async fn api_checker(req: &Request, token: Bearer) -> Option<User> {
    let server_key = req.data::<ServerKey>().unwrap();
    VerifyWithKey::<User>::verify_with_key(token.token.as_str(), server_key).ok()
}

#[derive(Object, Deserialize)]
struct AuthenticationRequest {
    email: Option<String>,
    password: Option<String>
}

#[derive(ApiResponse)]
enum AuthenticationResponse {
    #[oai(status = 200)]
    Token(PlainText<String>),

    #[oai(status = 400)]
    WrongCredentials(PlainText<String>),

    #[oai(status = 404)]
    NotFound(PlainText<String>)
}

#[OpenApi(tag = "ApiTags::Auth")]
impl AuthApi {
    #[oai(path = "/auth", method = "post")]
    async fn authenticate(&self, pool: Data<&Pool<Postgres>>, server_key: Data<&ServerKey>, req: Json<AuthenticationRequest>) -> Result<AuthenticationResponse> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", req.email).fetch_one(*pool).await;

        match user {
            Ok(user_data) => {
                match PasswordHash::new(&user_data.password) {
                    Ok(p) => {
                        if Argon2::default().verify_password(req.password.clone().expect("Unable to verify the given password").as_bytes(), &p).is_err() {
                            return Ok(AuthenticationResponse::WrongCredentials(PlainText("Wrong credentials provided".to_string())))
                        }

                        let token = user_data.sign_with_key(server_key.0).map_err(InternalServerError)?;
                        Ok(AuthenticationResponse::Token(PlainText(token)))
                    }
                    Err(_) => Ok(AuthenticationResponse::WrongCredentials(PlainText("Unable to verify the given password".to_string())))
                }
            }
            Err(_) => Ok(AuthenticationResponse::NotFound(PlainText(format!("No user matching the given credentials has been found"))))
        }
    }
}