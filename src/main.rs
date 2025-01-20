use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        LazyLock, RwLock,
    },
    time::Duration,
};

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing, Router,
};
use ffmonitor::{Event, Monitor, MonitorNotification, MonitorUpdate};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

static LATEST_UPDATE: LazyLock<RwLock<Option<MonitorUpdate>>> = LazyLock::new(|| RwLock::new(None));
static CONNECTED: AtomicBool = AtomicBool::new(true);

fn monitor_notification_callback(notification: MonitorNotification) {
    println!("Monitor: {:?}", notification);
    match notification {
        MonitorNotification::Connected => CONNECTED.store(true, Ordering::Relaxed),
        MonitorNotification::Disconnected => CONNECTED.store(false, Ordering::Relaxed),
        MonitorNotification::Updated(update) => {
            // Filter out all events except player events
            let mut filtered_update = MonitorUpdate::default();
            for event in update.get_events() {
                if let Event::Player(pe) = event {
                    filtered_update.add_event(Event::Player(pe));
                }
            }

            let mut latest_update = LATEST_UPDATE.write().unwrap();
            latest_update.replace(filtered_update);
        }
    }
}

#[tokio::main]
async fn main() {
    let _monitor =
        Monitor::new_with_callback("127.0.0.1:8003", Box::new(monitor_notification_callback))
            .unwrap();

    let client_dir = PathBuf::from("client");
    let app = Router::new()
        .fallback_service(ServeDir::new(client_dir).append_index_html_on_directories(true))
        .route("/ws", routing::any(ws_handler));

    let http_addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(http_addr).await.unwrap();
    println!("Serving at http://{}", http_addr);
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    println!("WebSocket connection requested");
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // If we can't ping the client, drop 'em
    if socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_err()
    {
        return;
    }

    println!("Client connected");

    let mut last_connected = true;
    loop {
        let latest_update = LATEST_UPDATE.read().unwrap().clone();
        let currently_connected = CONNECTED.load(Ordering::Relaxed);
        let msg = {
            if last_connected && !currently_connected {
                last_connected = false;
                Some("dc".to_string())
            } else if !last_connected && currently_connected {
                last_connected = true;
                Some("rc".to_string())
            } else if !currently_connected {
                None
            } else {
                latest_update.map(|update| format!("update\n{}", update))
            }
        };

        if let Some(msg) = msg {
            println!("Sending: {}", msg);
            if socket.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    println!("Client disconnected");
}
