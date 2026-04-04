use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::{mpsc, watch, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

// --- Types ---

struct ClientInfo {
    color_index: u32,
    join_order: u32,
    sender: mpsc::UnboundedSender<String>,
}

struct RelayState {
    clients: HashMap<String, ClientInfo>,
    host_id: Option<String>,
    last_game_state: Option<String>,
    next_color_index: u32,
    join_counter: u32,
}

type SharedState = Arc<Mutex<RelayState>>;

#[derive(Clone)]
struct AppState {
    relay: SharedState,
    index_html: PathBuf,
    state_watch: watch::Sender<Option<String>>,
}

#[derive(Deserialize)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct WsOutMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: serde_json::Value,
}

impl WsOutMessage {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// --- HTTP Handlers ---

async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::fs::read_to_string(&state.index_html).await {
        Ok(contents) => (
            [
                (axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (
                    axum::http::header::CONTENT_SECURITY_POLICY,
                    "default-src * 'self' 'unsafe-inline' 'unsafe-eval' data: blob: ws: wss: http: https:; img-src * data: blob: 'self'; media-src * 'self' http: https:; connect-src * ws: wss: http: https:;",
                ),
            ],
            axum::response::Html(contents),
        )
            .into_response(),
        Err(_) => (
            axum::http::StatusCode::NOT_FOUND,
            "index.html not found",
        )
            .into_response(),
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.relay, state.state_watch))
}

// --- WebSocket Connection Handler ---

async fn handle_socket(socket: WebSocket, relay: SharedState, state_watch: watch::Sender<Option<String>>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create a channel for sending control messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Assign an ID to this connection and determine role
    let (player_id, is_host) = {
        let mut state = relay.lock().await;
        let id = format!("conn_{}", state.join_counter);
        let color_index = state.next_color_index % 4;
        state.next_color_index += 1;
        let join_order = state.join_counter;
        state.join_counter += 1;

        state.clients.insert(
            id.clone(),
            ClientInfo {
                color_index,
                join_order,
                sender: tx.clone(),
            },
        );

        // Determine role
        let host = if state.host_id.is_none() {
            state.host_id = Some(id.clone());
            let role_msg = WsOutMessage {
                msg_type: "role".to_string(),
                data: serde_json::json!({
                    "role": "host",
                    "colorIndex": color_index,
                    "playerId": id
                }),
            };
            let _ = tx.send(role_msg.to_json());
            true
        } else {
            let role_msg = WsOutMessage {
                msg_type: "role".to_string(),
                data: serde_json::json!({
                    "role": "guest",
                    "colorIndex": color_index,
                    "playerId": id
                }),
            };
            let _ = tx.send(role_msg.to_json());

            // Send cached game state if available (raw "state" message — guest handles it the same as "current-state")
            if let Some(ref game_state) = state.last_game_state {
                let _ = tx.send(game_state.clone());
            }

            // Notify host about the new player
            if let Some(ref host_id) = state.host_id {
                if let Some(host) = state.clients.get(host_id) {
                    let join_msg = WsOutMessage {
                        msg_type: "player-joined".to_string(),
                        data: serde_json::json!({
                            "playerId": id,
                            "colorIndex": color_index
                        }),
                    };
                    let _ = host.sender.send(join_msg.to_json());
                }
            }
            false
        };

        (id, host)
    };

    // Subscribe to the watch channel for game state broadcasts (guests only —
    // the host generates state and doesn't need to read it back)
    let mut state_rx = if is_host {
        None
    } else {
        Some(state_watch.subscribe())
    };

    // Spawn a task to forward control messages and game state to the WebSocket
    let send_task = tokio::spawn(async move {
        loop {
            if let Some(ref mut watch_rx) = state_rx {
                // Guest path: prioritize state delivery over control messages
                tokio::select! {
                    biased;
                    result = watch_rx.changed() => {
                        if result.is_err() { break; }
                        let val = watch_rx.borrow_and_update().clone();
                        if let Some(json) = val {
                            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    msg = rx.recv() => {
                        match msg {
                            Some(m) => {
                                if ws_sender.send(Message::Text(m.into())).await.is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                }
            } else {
                // Host path: no watch subscription, only control messages
                match rx.recv().await {
                    Some(m) => {
                        if ws_sender.send(Message::Text(m.into())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    });

    // Process incoming messages
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                let text_str: &str = &text;
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(text_str) {
                    handle_message(&relay, &player_id, ws_msg, text_str, &state_watch).await;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Client disconnected — clean up
    handle_disconnect(&relay, &player_id).await;

    // Abort the send task
    send_task.abort();
}

async fn handle_message(relay: &SharedState, sender_id: &str, msg: WsMessage, raw_text: &str, state_watch: &watch::Sender<Option<String>>) {
    let mut state = relay.lock().await;

    match msg.msg_type.as_str() {
        "input" => {
            // Guest sends input -> relay to host as guest-input
            if let Some(ref host_id) = state.host_id {
                if sender_id != host_id {
                    if let Some(host) = state.clients.get(host_id) {
                        let mut data = msg.data.unwrap_or(serde_json::Value::Null);
                        if let Some(obj) = data.as_object_mut() {
                            obj.insert(
                                "playerId".to_string(),
                                serde_json::Value::String(sender_id.to_string()),
                            );
                        }
                        let out = WsOutMessage {
                            msg_type: "guest-input".to_string(),
                            data,
                        };
                        let _ = host.sender.send(out.to_json());
                    }
                }
            }
        }
        "state" => {
            // Host sends state -> cache raw text and forward directly (no re-serialization)
            if state.host_id.as_deref() == Some(sender_id) {
                state.last_game_state = Some(raw_text.to_string());
                drop(state);
                let _ = state_watch.send(Some(raw_text.to_string()));
                return;
            }
        }
        _ => {}
    }
}

async fn handle_disconnect(relay: &SharedState, disconnected_id: &str) {
    let mut state = relay.lock().await;

    state.clients.remove(disconnected_id);

    let was_host = state.host_id.as_deref() == Some(disconnected_id);

    if was_host {
        state.host_id = None;

        // Promote the client with the lowest join_order
        let new_host = state
            .clients
            .iter()
            .min_by_key(|(_, info)| info.join_order)
            .map(|(id, _)| id.clone());

        if let Some(ref new_host_id) = new_host {
            state.host_id = Some(new_host_id.clone());

            // Send promote-to-host to the new host
            if let Some(new_host_client) = state.clients.get(new_host_id) {
                // last_game_state is the raw '{"type":"state","data":{...}}' message.
                // Extract just the inner "data" field for the promote payload.
                let cached_state = state.last_game_state.as_deref()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                    .and_then(|v| v.get("data").cloned());
                let promote_msg = WsOutMessage {
                    msg_type: "promote-to-host".to_string(),
                    data: serde_json::json!({
                        "state": cached_state
                    }),
                };
                let _ = new_host_client.sender.send(promote_msg.to_json());
            }

            // Notify all other clients about the host change
            let host_changed_json = WsOutMessage {
                msg_type: "host-changed".to_string(),
                data: serde_json::json!({
                    "newHostId": new_host_id
                }),
            }
            .to_json();

            for (id, client) in &state.clients {
                if id != new_host_id {
                    let _ = client.sender.send(host_changed_json.clone());
                }
            }
        }
    } else {
        // Notify host that a player left
        if let Some(ref host_id) = state.host_id {
            if let Some(host) = state.clients.get(host_id) {
                let leave_msg = WsOutMessage {
                    msg_type: "player-left".to_string(),
                    data: serde_json::json!({
                        "playerId": disconnected_id
                    }),
                };
                let _ = host.sender.send(leave_msg.to_json());
            }
        }
    }
}

// --- Relay Server Startup ---

async fn start_relay_server(sounds_dir: PathBuf, index_html: PathBuf) {
    let relay = Arc::new(Mutex::new(RelayState {
        clients: HashMap::new(),
        host_id: None,
        last_game_state: None,
        next_color_index: 0,
        join_counter: 0,
    }));

    let (state_watch, _) = watch::channel::<Option<String>>(None);

    let app_state = AppState {
        relay,
        index_html,
        state_watch,
    };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/ws", get(ws_handler))
        .nest_service("/sounds", ServeDir::new(sounds_dir))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3800")
        .await
        .expect("Failed to bind relay server to port 3800");

    axum::serve(listener, app).await.ok();
}

// --- Tauri Entry Point ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Resolve resource paths
            let resource_dir = app
                .path()
                .resource_dir()
                .expect("Failed to resolve resource directory");

            // In dev mode, resource_dir points to target/debug which may have stale copies.
            // Prefer the source files if they exist (dev), fall back to resource_dir (production).
            let src_tauri_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let dev_index = src_tauri_dir.join("index.html");
            let dev_sounds = src_tauri_dir.join("sounds");
            let index_html = if dev_index.exists() { dev_index } else { resource_dir.join("index.html") };
            let sounds_dir = if dev_sounds.exists() { dev_sounds } else { resource_dir.join("sounds") };

            // Spawn the relay server before the window loads
            tauri::async_runtime::spawn(async move {
                start_relay_server(sounds_dir, index_html).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
