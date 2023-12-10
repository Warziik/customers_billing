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
    let server_port = 8080;
    let database_url = env::var("DATABASE_URL").expect("The DATABASE_URL env variable must be provided.");

    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    let api_service = OpenApiService::new(Api, "API", env!("CARGO_PKG_VERSION")).server(format!("http://localhost:{}", server_port));
    let ui = api_service.swagger_ui();
    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .data(pool);

    Server::new(TcpListener::bind(format!("127.0.0.1:{}", server_port)))
        .run(app)
        .await.unwrap();
}