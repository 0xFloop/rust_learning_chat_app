use axum_macros::debug_handler;
use rand::prelude::*;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[allow(unused_imports)]
use axum::{
    extract::State,
    extract::{Json, Path, Query},
    response::Html,
    routing::get,
    routing::post,
    Router,
};

use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use serde_json::{json, Value};
use tokio::sync::broadcast;

#[derive(Debug)]
struct AppState {
    user_names: Mutex<HashSet<String>>,
    snd: broadcast::Sender<Message>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    pub room_number: u32,
    pub sender_username: String,
    pub message_text: String,
}

#[tokio::main]
async fn main() {
    let (snd, _rec) = broadcast::channel::<Message>(16);
    let user_names = HashSet::new();

    let app_state = Arc::new(AppState {
        user_names: Mutex::new(user_names),
        snd: snd,
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/send", post(send_message))
        .route("/api/receive", post(receive_messages))
        .with_state(app_state);

    let listner = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listner, app).await.unwrap();
}

//look up long lived server ocnnections with axum
//look up building ui in rust

async fn index() -> Html<&'static str> {
    //return web page for sending
    return Html("../html/index.html");
}
#[debug_handler]
async fn send_message(State(state): State<Arc<AppState>>, Json(payload): Json<Message>) {
    //endpoint for receiving new message payloads and passing them the eveyone listening for them
    let mut rng = rand::thread_rng();
    let rand_num: u32 = rng.gen();

    println!("{:?}", payload);
    let _sent = state.snd.send(Message {
        room_number: payload.room_number,
        sender_username: payload.sender_username,
        message_text: payload.message_text,
    });
}

async fn receive_messages(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> String {
    //endpoint for subscribing to new messages coming in
    let mut un = state.snd.subscribe();
    loop {
        let msg = un.recv().await.unwrap().message_text;
    }
}
