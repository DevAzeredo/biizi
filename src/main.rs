use axum::{
    extract::{connect_info::ConnectInfo, Path, State}, response::IntoResponse, routing::{any, post}, Json, Router
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod websocket {
    pub mod websocket;
}

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

// Configura as rotas
fn create_router(ws_manager: WebSocketManager) -> Router {
    // Clona o WebSocketManager para as rotas
    let ws_manager_clone = ws_manager.clone();

    Router::new()
        // Rota WebSocket
        .route(
            "/ws",
            any(move |ws, user_agent, addr: ConnectInfo<SocketAddr>| {
                ws_manager_clone.clone().ws_handler(ws, user_agent, addr)
            }),
        )
        // Rota para enviar mensagens
        .route("/send/:addr", post(send_message_handler)).with_state(ws_manager)
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

#[tokio::main]
async fn main() {
    // Inicializa logs e tracing
    init_tracing();

    let ws_manager = WebSocketManager::new();

    let app = create_router(ws_manager);

    let listener = TcpListener::bind(SERVER_ADDR).await.unwrap();
    tracing::info!("🚀 Server listening on {}", SERVER_ADDR);

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
