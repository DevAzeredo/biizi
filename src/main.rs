use axum::{
    extract::{connect_info::ConnectInfo, Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};

use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, AsyncConnection};
use domain::models::{NewCompany, NewEmployee, NewJobOpportunity, NewUser, User};
use infrastructure::auth::{self, Auth, SignInData};
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

async fn get_employee(
    State(pool): State<Pool>,
    Extension(user): Extension<User>,
) -> Result<Json<Employee>, StatusCode> {
    let mut conn = pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user.employeeid {
        Some(id) => match Service::find_employee(&mut conn, &id).await {
            Ok(employee) => Ok(Json(employee)),
            Err(_) => Err(StatusCode::NOT_FOUND),
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_company(
    State(pool): State<Pool>,
    Extension(user): Extension<User>,
) -> Result<Json<Company>, StatusCode> {
    let mut conn = pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user.companyid {
        Some(id) => match Service::find_company(&mut conn, &id).await {
            Ok(company) => Ok(Json(company)),
            Err(_) => Err(StatusCode::NOT_FOUND),
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_employee(
    State(pool): State<Pool>,
    Extension(user): Extension<User>,
    Json(employee): Json<NewEmployee>,
) -> Result<Json<Employee>, StatusCode> {
    let mut conn = pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let res = Service::add_employee(&mut conn, employee, user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(res)
}

pub async fn register_user(
    State(pool): State<Pool>,
    Json(new_user): Json<NewUser>,
) -> Result<String, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let token = Service::register_user(&mut conn, new_user)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(token)
}

async fn login(
    State(pool): State<Pool>,
    Json(credentials): Json<SignInData>,
) -> Result<String, StatusCode> {
    let mut conn = pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = auth::sign_in(&mut conn, credentials)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(token)
}

async fn upload_company_logo(
    State(pool): State<Pool>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    let company_id = match user.companyid {
        Some(id) => id,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "logo" {
            let data = field
                .bytes()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let file_name = format!("{}.png", company_id);
            let file_path = format!("./assets/logos/{}", file_name);

            // Cria o diretório se não existir
            tokio::fs::create_dir_all("./assets/logos")
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Salva o arquivo
            tokio::fs::write(&file_path, &data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Atualiza o banco de dados
            let mut conn = pool
                .get()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let _ = Service::update_company_logo(&mut conn, company_id, file_name.clone())
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(format!("Logo uploaded successfully: {}", file_name).into_response());
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

pub async fn list_job_opportunities(
    State(pool): State<Pool>,
    Path(company_id): Path<i64>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut conn = pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = Service::get_job_opportunities_with_company(&mut conn, company_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

async fn create_company(
    State(pool): State<Pool>,
    Extension(user): Extension<User>,
    Json(company): Json<NewCompany>,
) -> Result<Json<Company>, StatusCode> {
    let mut conn = pool
        .get()
        .await
        .map_err(internal_error)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let res = Service::add_company(&mut conn, company, user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(res)
}

async fn create_job(
    State(pool): State<Pool>,
    Extension(user): Extension<User>,
    Json(job): Json<NewJobOpportunity>,
) -> Result<Json<JobOpportunity>, StatusCode> {
    let mut conn = pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let res = Service::add_job_opportunity(&mut conn, job, user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(res)
}
type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub const MIGRATIONS: diesel_async_migrations::EmbeddedMigrations = diesel_async_migrations::embed_migrations!();
async fn run_migrations(url: impl AsRef<str>) -> anyhow::Result<()> {
    let mut conn = AsyncPgConnection::establish(url.as_ref()).await?;
    MIGRATIONS.run_pending_migrations(&mut conn).await?;
    Ok(())
}

async fn create_router(ws_manager: WebSocketManager) -> Router {
    let ws_manager_clone = ws_manager.clone();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    run_migrations(db_url.clone()).await.unwrap();
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
            post(create_employee).route_layer(axum::middleware::from_fn_with_state(
                pool.clone(),
                Auth::authorize,
            )),
        )
        .route(
            "/employees",
            get(get_employee).route_layer(axum::middleware::from_fn_with_state(
                pool.clone(),
                Auth::authorize,
            )),
        )
        .route(
            "/companies",
            post(create_company).route_layer(axum::middleware::from_fn_with_state(
                pool.clone(),
                Auth::authorize,
            )),
        )
        .route(
            "/companies",
            get(get_company).route_layer(axum::middleware::from_fn_with_state(
                pool.clone(),
                Auth::authorize,
            )),
        )
        .route(
            "/companies/upload-logo",
            post(upload_company_logo).route_layer(axum::middleware::from_fn_with_state(
                pool.clone(),
                Auth::authorize,
            )),
        )
        .route(
            "/jobs",
            post(create_job).route_layer(axum::middleware::from_fn_with_state(
                pool.clone(),
                Auth::authorize,
            )),
        )
        .route("/jobs/:company_id", get(list_job_opportunities))
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
