pub mod messages;
mod ws_handler;

pub use messages::{GameEvent, MarketItem, PlayerInput};

use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::{broadcast, mpsc};
use tower_http::services::ServeDir;

use messages::ServerMsg;
use ws_handler::handle_socket;

const MAX_PLAYERS: u8 = 8;

#[derive(Clone)]
struct AppState {
    player_count: Arc<AtomicU8>,
    input_tx: mpsc::Sender<PlayerInput>,
    event_tx: broadcast::Sender<GameEvent>,
}

pub async fn run(
    input_tx: mpsc::Sender<PlayerInput>,
    event_tx: broadcast::Sender<GameEvent>,
) {
    let state = AppState {
        player_count: Arc::new(AtomicU8::new(0)),
        input_tx,
        event_tx,
    };

    let app = Router::new()
        .route("/ws", get(ws_upgrade))
        .nest_service("/", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("[server] listening on :3000");

    axum::serve(listener, app).await.unwrap();
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let count = state.player_count.fetch_add(1, Ordering::SeqCst);

    if count >= MAX_PLAYERS {
        state.player_count.fetch_sub(1, Ordering::SeqCst);
        return ws.on_upgrade(|mut socket| async move {
            let msg = ServerMsg::Full;
            let json = serde_json::to_string(&msg).unwrap();
            use axum::extract::ws::Message;
            let _ = socket.send(Message::Text(json.into())).await;
        });
    }

    let player_id    = count;
    let input_tx     = state.input_tx.clone();
    let event_rx     = state.event_tx.subscribe();
    let player_count = state.player_count.clone(); // Arc — คืน slot เมื่อ disconnect

    println!("[server] player {} connected (slots used: {})", player_id, count + 1);

    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, player_id, input_tx, event_rx).await;
        // handle_socket return = socket ปิดแล้ว → คืน slot
        player_count.fetch_sub(1, Ordering::SeqCst);
        println!("[server] slot {} freed (slots used: {})",
            player_id, player_count.load(Ordering::SeqCst));
    })
}
