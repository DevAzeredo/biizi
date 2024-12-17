use axum::{
    extract::{connect_info::ConnectInfo, Path, State}, http::StatusCode, response::IntoResponse, routing::{any, get, post}, Json, Router
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod infrastructure {
    pub mod repositories;
    pub mod schema;
}
mod websocket { 
    pub mod websocket;
}
mod domain {
    pub mod models;
}
mod application {
    pub mod service;
}
use self::domain::models::{Company, Employee, JobOpportunity};
use self::application::service::Service;

use crate::websocket::websocket::WebSocketManager;
use axum::extract::ws::Message;

// Constantes
const SERVER_ADDR: &str = "0.0.0.0:9854";
const ASSETS_DIR: &str = "assets";

// Inicializa o Tracing (Logging)
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub async fn send_message_handler(
    Path(addr): Path<String>,
    State(ws_manager): State<WebSocketManager>,
 ) -> impl IntoResponse {
     if let Ok(client_addr) = addr.parse::<SocketAddr>() {
         let msg = Message::Text("Hello, World!".to_string());
         ws_manager.send_to_client(client_addr, msg).await;
         Json("status: Message sent".to_string()).into_response()
     } else {
         Json("error: Invalid address format".to_string()).into_response()
     }
 }

  async fn create_employee(
    State(pool): State<Pool>,
    Json(employee): Json<Employee>,
) -> Result<Json<Employee>, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
  let res =   Service::add_employee(&mut conn, employee).await.map_err(internal_error)?;
 Ok(res)
}

 async fn create_company(
    State(pool): State<Pool>,
    Json(company): Json<Company>,
) -> Result<Json<Company>, (StatusCode, String)>  {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let res = Service::add_company(&mut conn, company).await.map_err(internal_error)?;
    Ok(res)
}

 async fn create_job(
    State(pool): State<Pool>,
    Json(job): Json<JobOpportunity>,
)-> Result<Json<JobOpportunity>, (StatusCode, String)>  {
    let mut conn = pool.get().await.map_err(internal_error)?;
   let res = Service::add_job_opportunity(&mut conn, job)
        .await.map_err(internal_error)?;
    Ok(res)
}
type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
// Configura as rotas
async fn create_router(ws_manager: WebSocketManager) -> Router {
    // Clona o WebSocketManager para as rotas
    let ws_manager_clone = ws_manager.clone();
    // bd
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = bb8::Pool::builder().build(config).await.unwrap();

    Router::new()
        // Rota WebSocket
        .route(
            "/ws",
            get(move |ws, user_agent, addr: ConnectInfo<SocketAddr>| {
                ws_manager_clone.clone().ws_handler(ws, user_agent, addr)
            }),
        )
        // Rota para enviar mensagens
        .route("/send/:addr", post(send_message_handler)).with_state(ws_manager)
        .route("/employees", post(create_employee))
        .route("/companies", post(create_company))
        .route("/jobs", post(create_job))
        .with_state(pool)
        // Servir arquivos estáticos
        .fallback_service(
            ServeDir::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ASSETS_DIR))
                .append_index_html_on_directories(true),
        )
        // Middleware de logging
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
}
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}


#[tokio::main]
async fn main() {
    // Inicializa logs e tracing
    init_tracing();

    let ws_manager = WebSocketManager::new();

    let app = create_router(ws_manager);

    let listener = TcpListener::bind(SERVER_ADDR).await.unwrap();
    tracing::info!("🚀 Server listening on {}", SERVER_ADDR);

    axum::serve(listener, app.await.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
