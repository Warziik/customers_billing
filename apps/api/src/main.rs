use std::env;
use hmac::{Hmac, Mac};
use poem::{EndpointExt, listener::TcpListener, Route, Server};
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
use sha2::Sha256;
use crate::auth::{AuthApi, AuthProvider, SERVER_KEY};
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
    dotenvy::dotenv();
    // dotenv!()
    let server_port = 8080;

    let pool = sqlx::PgPool::connect("postgres://postgres:root@localhost:5432/postgres").await.unwrap();
    let endpoints = (Api, AuthApi, UserApi);
    let api_service = OpenApiService::new(endpoints, "API", env!("CARGO_PKG_VERSION")).server(format!("http://localhost:{}", server_port));
    let ui = api_service.swagger_ui();
    let server_key = Hmac::<Sha256>::new_from_slice(SERVER_KEY).expect("Expected a SERVER_KEY for JWT Token");
    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .data(pool)
        .data(server_key);

    Server::new(TcpListener::bind(format!("127.0.0.1:{}", server_port)))
        .run(app)
        .await
}