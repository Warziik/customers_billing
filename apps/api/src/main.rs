use std::env;
use std::path::Path;
use hmac::{Hmac, Mac};
use poem::{EndpointExt, listener::TcpListener, Route, Server};
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
use sha2::Sha256;
use crate::auth::{AuthApi, AuthProvider};
use crate::user::{UserApi};

mod auth;
mod user;

struct Api;

#[OpenApi]
impl Api {
    /// Hello World
    #[oai(path = "/", method = "get")]
    async fn index(&self, auth: AuthProvider) -> PlainText<String> {
        PlainText(format!("Hello world! {}", auth.0.firstname))
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if Path::new("apps/api/.env").exists() {
        dotenvy::from_path(Path::new("apps/api/.env")).expect(".env file not found");
    } else {
        dotenvy::dotenv().expect(".env file not found");
    }

    let database_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set in the .env file");
    let jwt_secret_var: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set in the .env file");
    let jwt_secret: &[u8] = &jwt_secret_var.as_bytes();
    let server_port: String = env::var("SERVER_PORT").expect("SERVER_PORT must be set in the .env file");

    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    let api_service = OpenApiService::new((Api, AuthApi, UserApi), "API", env!("CARGO_PKG_VERSION")).server(format!("http://localhost:{}", &server_port));
    let ui = api_service.swagger_ui();
    let server_key = Hmac::<Sha256>::new_from_slice(&jwt_secret).expect("Expected a JWT_SECRET for JWT Token");
    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .data(pool)
        .data(server_key);

    Server::new(TcpListener::bind(format!("127.0.0.1:{}", &server_port)))
        .run(app)
        .await
}