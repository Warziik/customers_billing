use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use chrono::{DateTime, Utc};
use poem::{Result};
use poem::web::Data;
use poem_openapi::{Object, OpenApi, ApiResponse};
use poem_openapi::payload::{Json, PlainText};
use poem_openapi::param::Path;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use crate::auth::AuthProvider;

pub struct UserApi;

#[derive(Object, FromRow, Debug, Deserialize, Serialize)]
pub struct User {
    #[oai(read_only)]
    pub id: i32,

    #[oai(validator(max_length = 64))]
    pub firstname: String,

    #[oai(validator(max_length = 64))]
    pub lastname: String,

    #[oai(validator(max_length = 64))]
    pub email: String,

    #[oai(validator(max_length = 64))]
    #[oai(skip)]
    pub password: String,

    #[oai(read_only)]
    pub created_at: DateTime<Utc>,

    #[oai(read_only)]
    pub updated_at: Option<DateTime<Utc>>
}

#[derive(ApiResponse)]
enum GetUserResponse {
    #[oai(status = 200)]
    User(Json<User>),

    #[oai(status = 404)]
    NotFound(PlainText<String>)
}

#[OpenApi]
impl UserApi {
    #[oai(path = "/users", method = "post")]
    async fn create_user(&self, pool: Data<&Pool<Postgres>>, data: Json<User>) -> Result<GetUserResponse> {
        let salt = SaltString::generate(&mut OsRng);
        let password = Argon2::default().hash_password(data.password.clone().as_bytes(), &salt).expect("Unable to hash password").to_string();

        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (firstname, lastname, email, password) VALUES($1, $2, $3, $4) RETURNING *",
            &data.firstname,
            &data.lastname,
            &data.email,
            password
        ).fetch_one(*pool).await;

        match user {
            Ok(user) => Ok(GetUserResponse::User(Json(user))),
            Err(_) => Ok(GetUserResponse::NotFound(PlainText(format!("Unable to create the user"))))
        }
    }

    #[oai(path = "/users/:id", method = "get")]
    async fn get_user(&self, pool: Data<&Pool<Postgres>>, auth: AuthProvider, id: Path<i32>) -> Result<GetUserResponse> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id.0
        ).fetch_one(*pool).await;

        match user {
            Ok(user) => Ok(GetUserResponse::User(Json(user))),
            Err(_) => Ok(GetUserResponse::NotFound(PlainText(format!("User with id {} not found", id.0))))
        }
    }

    #[oai(path = "/me", method = "get")]
    async fn get_logged_user(&self, pool: Data<&Pool<Postgres>>, auth: AuthProvider) -> Result<GetUserResponse> {
        Ok(GetUserResponse::User(Json(auth.0)))
    }
}