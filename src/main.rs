use axum::{
    extract::{connect_info::ConnectInfo, Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use domain::models::{NewCompany, NewEmployee, NewJobOpportunity, NewUser, User};
use infrastructure::auth::{self, authorize, AuthenticatedUser};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod infrastructure {
    pub mod auth;
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
use self::application::service::Service;
use self::domain::models::{Company, Employee, JobOpportunity};

use crate::websocket::websocket::WebSocketManager;
use axum::extract::ws::Message;

const SERVER_ADDR: &str = "0.0.0.0:9854";
const ASSETS_DIR: &str = "assets";

// Inicializa o Logging
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
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
    Json(employee): Json<NewEmployee>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Employee>, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let res = Service::add_employee(&mut conn, employee, user)
        .await
        .map_err(internal_error)?;
    Ok(res)
}

pub async fn register_user(
    State(pool): State<Pool>,
    Json(payload): Json<NewUser>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let hashed_password = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash password".to_string(),
        )
    })?;

    let new_user = NewUser {
        login: payload.login.clone(),
        password: hashed_password,
    };

    let token = Service::register_user(&mut conn, new_user)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(token)
}

async fn login(
    State(pool): State<Pool>,
    Json(credentials): Json<NewUser>,
) -> Result<String, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let user = Service::authenticate(&mut conn, &credentials.login, &credentials.password)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let token = Service::generate_jwt(&user).map_err(internal_error)?;

    Ok(token)
}

async fn create_company(
    State(pool): State<Pool>,
    Json(company): Json<NewCompany>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Company>, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let res = Service::add_company(&mut conn, company, user)
        .await
        .map_err(internal_error)?;
    Ok(res)
}

async fn create_job(
    State(pool): State<Pool>,
    Json(job): Json<NewJobOpportunity>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<JobOpportunity>, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let res = Service::add_job_opportunity(&mut conn, job, user)
        .await
        .map_err(internal_error)?;
    Ok(res)
}

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
async fn create_router(ws_manager: WebSocketManager) -> Router {
    let ws_manager_clone = ws_manager.clone();
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
        .route("/send/:addr", post(send_message_handler))
        .with_state(ws_manager)
        .route(
            "/employees",
            post(create_employee).layer(axum::middleware::from_fn(authorize)),
        )
        .route(
            "/companies",
            post(create_company).layer(axum::middleware::from_fn(authorize)),
        )
        .route(
            "/jobs",
            post(create_job).layer(axum::middleware::from_fn(authorize)),
        )
        .route("/login", post(login))
        .route("/register", post(register_user))
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

    axum::serve(
        listener,
        app.await
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
