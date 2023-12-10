use std::env;
use poem::{EndpointExt, listener::TcpListener, Route, Server};
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};

struct Api;

#[OpenApi]
impl Api {
    /// Hello World
    #[oai(path = "/", method = "get")]
    async fn index(&self) -> PlainText<&'static str> {
        PlainText("Hello world!")
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool = sqlx::PgPool::connect(env::var("DATABASE_URL").expect("The DATABASE_URL env variable must be provided.")).await.unwrap();
    let api_service = OpenApiService::new(Api, "API", "0.1.0").server("http://localhost:8080");
    let ui = api_service.swagger_ui();
    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .data(pool);

    Server::new(TcpListener::bind("127.0.0.1:8080"))
        .run(app)
        .await.unwrap();
}