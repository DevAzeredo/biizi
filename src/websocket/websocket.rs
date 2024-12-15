use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::connect_info::ConnectInfo,
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use futures::{stream::SplitSink, StreamExt, SinkExt};
use headers::UserAgent;
use std::{
    collections::HashMap,
    net::SocketAddr,
    ops::ControlFlow,
    sync::Arc,
};
use tokio::sync::RwLock;

// Clients type definition
type Clients = Arc<RwLock<HashMap<SocketAddr, SplitSink<WebSocket, Message>>>>;

/// A WebSocket Manager to handle connected clients and messages.
#[derive(Clone, Default)]
pub struct WebSocketManager {
    clients: Clients,
}

impl WebSocketManager {
    /// Create a new WebSocketManager instance.
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new client to the manager.
    async fn add_client(&self, addr: SocketAddr, sender: SplitSink<WebSocket, Message>) {
        self.clients.write().await.insert(addr, sender);
        println!("Client {addr} added");
    }

    /// Remove a client from the manager.
    async fn remove_client(&self, addr: SocketAddr) {
        self.clients.write().await.remove(&addr);
        println!("Client {addr} removed");
    }

    /// Send a message to a specific client.
    pub async fn send_to_client(&self, addr: SocketAddr, msg: Message) {
        let mut clients_lock = self.clients.write().await;
        if let Some(sender) = clients_lock.get_mut(&addr) {
            if let Err(e) = sender.send(msg).await {
                println!("Error sending message to {addr}: {e}");
            }
        } else {
            println!("Client {addr} not found");
        }
    }

    /// WebSocket handler for incoming HTTP connections.
    pub async fn ws_handler(
        self,
        ws: WebSocketUpgrade,
        user_agent: Option<TypedHeader<UserAgent>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ) -> impl IntoResponse {
        let user_agent = user_agent
            .map(|TypedHeader(ua)| ua.to_string())
            .unwrap_or_else(|| "Unknown browser".to_string());

        println!("`{user_agent}` at {addr} connected.");
        ws.on_upgrade(move |socket| self.clone().handle_socket(socket, addr))
    }

    /// Handle an upgraded WebSocket connection.
    async fn handle_socket(self, socket: WebSocket, addr: SocketAddr) {
        let (sender, mut receiver) = socket.split();

        self.add_client(addr, sender).await;

        // Handle incoming messages
        while let Some(Ok(msg)) = receiver.next().await {
            if Self::process_message(&msg, addr).is_break() {
                break;
            }
        }

        // Remove the client when disconnected
        self.remove_client(addr).await;
        println!("Client {addr} disconnected");
    }

    /// Process an incoming WebSocket message.
    fn process_message(msg: &Message, addr: SocketAddr) -> ControlFlow<(), ()> {
        match msg {
            Message::Text(t) => println!(">>> {addr} sent text: {t:?}"),
            Message::Binary(d) => println!(">>> {addr} sent binary ({} bytes)", d.len()),
            Message::Close(c) => {
                if let Some(cf) = c {
                    println!(
                        ">>> {addr} sent close: code {} reason `{}`",
                        cf.code, cf.reason
                    );
                } else {
                    println!(">>> {addr} sent a close message without details");
                }
                return ControlFlow::Break(());
            }
            Message::Ping(v) => println!(">>> {addr} sent ping: {v:?}"),
            Message::Pong(v) => println!(">>> {addr} sent pong: {v:?}"),
        }
        ControlFlow::Continue(())
    }
}