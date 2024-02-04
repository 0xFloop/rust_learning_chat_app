use core::time;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    thread,
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};

use serde::{Deserialize, Serialize};

use futures::{sink::SinkExt, stream::StreamExt};

#[allow(unused_imports)]
use serde_json::{json, Value};
use tokio::sync::broadcast;

#[derive(Debug)]
struct AppState {
    user_names: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    pub room_number: u32,
    pub sender_username: String,
    pub message_text: String,
}

#[tokio::main]
async fn main() {
    let (tx, _rec) = broadcast::channel::<String>(100);

    let app_state = Arc::new(AppState {
        user_names: Mutex::new(HashSet::new()),
        tx,
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/connect", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    //upgrades our connection to websocket and passes the new socket to `connect`
    println!("attempt to connect to server");
    ws.on_upgrade(move |socket| connect(socket, state))
}

async fn connect(ws: WebSocket, state: Arc<AppState>) {
    //sender is used to send new messages from the server to the connected client.
    //receiver is used to intake new messages from the connected client.
    //the names are from the perspective of the server, not the connected client.
    let (mut sender, mut receiver) = ws.split();

    let mut username = String::new();

    while let Some(Ok(user_name)) = receiver.next().await {
        if let Message::Text(name) = user_name {
            //you have to scope this locking of the mutex so that it is automatically unlocked when
            //the mutex is dropped. Otherwise the call could `return` and never unlock.
            {
                let mut connected_usernames = state.user_names.lock().unwrap();

                if !connected_usernames.contains(&name) {
                    connected_usernames.insert(name.to_owned());

                    username.push_str(&name);
                }
            }

            if !username.is_empty() {
                break;
            } else {
                let _ = sender
                    .send(Message::Text(String::from("Username Taken")))
                    .await;
                return;
            }
        }
    }

    //this creates a new subscription to all messages comming in from connected clients
    let mut rx = state.tx.subscribe();

    let msg = format!("{username} joined.");
    //this sends via the global sender to all websockets connected
    let _ = state.tx.send(msg);

    //this spawns a new thread for the new connection that awaits all new messages from
    //the global receiver and sends them to the connected client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    //this creates a clone of the global receiver that we can
    //send our own new messages to (hence tx or 'transmit')
    let tx = state.tx.clone();
    let name = username.clone();
    let mut receive_task = tokio::spawn(async move {
        //each time our local receiver instance gets a new message from our socket
        //we will broadcast that new message to the global receiver
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx.send(format!("{name}: {text}"));
        }
    });
    //if either of the two tasks finishes that means we have disconnected
    //so we can abort the other task and send a user left message.
    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }

    let msg = format!("{username} left.");
    let _ = state.tx.send(msg);

    state.user_names.lock().unwrap().remove(&username);
}

async fn index() -> &'static str {
    //return web page for messaging
    &"Hello world!"
}
