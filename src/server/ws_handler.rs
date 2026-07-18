use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};

use crate::server::messages::{ClientMsg, GameEvent, PlayerInput, ServerMsg};

pub async fn handle_socket(
    socket: WebSocket,
    player_id: u8,
    input_tx: mpsc::Sender<PlayerInput>,
    mut event_rx: broadcast::Receiver<GameEvent>,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Clone ไว้ส่ง disconnect notification หลัง socket หลุด
    let input_tx_dc = input_tx.clone();

    // ส่ง Joined message ทันทีที่ connect
    let joined = ServerMsg::Joined { player_id, slot: player_id };
    if let Ok(json) = serde_json::to_string(&joined) {
        let _ = ws_tx.send(Message::Text(json.into())).await;
    }

    // task A: game events → phone
    let send_task = tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(GameEvent::PlayerState { player_id: pid, hp, max_hp, score, coins, weapon_level, respawning, respawn_seconds })
                    if pid == player_id =>
                {
                    let msg = ServerMsg::State { hp, max_hp, score, coins, weapon_level, respawning, respawn_seconds };
                    let json = serde_json::to_string(&msg).unwrap();
                    if ws_tx.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Ok(GameEvent::Broadcast(text)) => {
                    let msg = ServerMsg::Event { msg: text };
                    let json = serde_json::to_string(&msg).unwrap();
                    if ws_tx.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Ok(GameEvent::SendMarket { player_id: pid, items })
                    if pid == player_id =>
                {
                    let msg = ServerMsg::Market { items };
                    let json = serde_json::to_string(&msg).unwrap();
                    if ws_tx.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });

    // task B: phone input → game
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            let text = match msg {
                Message::Text(t) => t,
                Message::Close(_) => break,
                _ => continue,
            };

            let client_msg: ClientMsg = match serde_json::from_str(&text) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[ws] player {} bad msg: {e}", player_id);
                    continue;
                }
            };

            match client_msg {
                ClientMsg::Input { joy, fire } => {
                    let _ = input_tx.send(PlayerInput {
                        player_id,
                        joy,
                        fire,
                        buy_item: None,
                        name: None,
                        disconnect: false,
                    }).await;
                }
                ClientMsg::Join { name } => {
                    println!("[ws] player {} joined as '{}'", player_id, name);
                    let _ = input_tx.send(PlayerInput {
                        player_id,
                        joy: [0.0, 0.0],
                        fire: false,
                        buy_item: None,
                        name: Some(name),
                        disconnect: false,
                    }).await;
                }
                ClientMsg::Buy { item } => {
                    let _ = input_tx.send(PlayerInput {
                        player_id,
                        joy: [0.0, 0.0],
                        fire: false,
                        buy_item: Some(item),
                        name: None,
                        disconnect: false,
                    }).await;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    // แจ้ง game ว่า player คนนี้ disconnect แล้ว
    let _ = input_tx_dc.send(PlayerInput {
        player_id,
        joy: [0.0, 0.0],
        fire: false,
        buy_item: None,
        name: None,
        disconnect: true,
    }).await;

    println!("[ws] player {} disconnected", player_id);
}
