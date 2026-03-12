mod game;
mod server;

use local_ip_address::local_ip;
use macroquad::prelude::*;
use tokio::sync::{broadcast, mpsc};

fn window_conf() -> Conf {
    Conf {
        window_title: "Starship Space".to_string(),
        window_width: 1600,
        window_height: 900,
        high_dpi: true,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // channels: phone input → game, game event → phones
    let (input_tx, input_rx) = mpsc::channel::<server::PlayerInput>(256);
    let (event_tx, _) = broadcast::channel::<server::GameEvent>(256);

    // detect local IP and build connection URL
    let host_url = match local_ip() {
        Ok(ip) => {
            let url = format!("http://{}:3000", ip);
            println!("[info] Players open: {}", url);
            url
        }
        Err(_) => {
            println!("[info] Players open: http://<YOUR-IP>:3000");
            "http://YOUR-IP:3000".to_string()
        }
    };

    // spawn Axum server on background thread
    let event_tx_server = event_tx.clone();
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .expect("tokio runtime")
            .block_on(server::run(input_tx, event_tx_server));
    });

    // run macroquad game loop (must stay on main thread)
    game::run(input_rx, event_tx, host_url).await;
}
