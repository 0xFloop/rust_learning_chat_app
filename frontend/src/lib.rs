use leptos::{leptos_dom::logging::console_log, *};
use wasm_bindgen::prelude::*;
use web_sys::{js_sys::Date, KeyboardEvent, MessageEvent, WebSocket};
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ChatMessage {
    msg: String,
    timestamp: u64,
}
#[component]
pub fn UsernameInput(
    messages_state: (ReadSignal<Vec<ChatMessage>>, WriteSignal<Vec<ChatMessage>>),
    joined_state: (ReadSignal<bool>, WriteSignal<bool>),
    ws_state: (
        ReadSignal<Option<WebSocket>>,
        WriteSignal<Option<WebSocket>>,
    ),
) -> impl IntoView {
    let (get_messages, send_new_message) = messages_state;
    let (_, set_joined) = joined_state;
    let join_chat = {
        move |event: KeyboardEvent| {
            let key = &event.key();
            if key == "Enter" {
                let username = event_target_value(&event);
                console_log(&username);
                // let username_string = username.node_value().unwrap();
                let ws = WebSocket::new("ws://localhost:3000/api/connect").unwrap();
                let ws_clone = ws.clone();
                let onopen_callback = Closure::once_into_js(move |e: MessageEvent| {
                    let data = e.data();
                    let (_, set_ws) = ws_state;
                    set_ws(Some(ws_clone.clone()));

                    let join_message = ws_clone.send_with_str(&username);
                });
                ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));

                let onmessage_callback =
                    Closure::<dyn Fn(MessageEvent)>::new(move |e: MessageEvent| {
                        if e.data().as_string().unwrap() != "Username Taken" {
                            set_joined(true);
                        }
                        let mut messages_clone = get_messages();

                        messages_clone.push(ChatMessage {
                            msg: e.data().as_string().unwrap(),
                            timestamp: Date::new_0().get_milliseconds() as u64,
                        });
                        send_new_message(messages_clone);
                    });
                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();
            }
        }
    };
    view! {
        <div>
        <h1> "Press 'Enter' to send "</h1>
            <input on:keydown=join_chat id="username" style="display:block; width:100px; box-sizing: border-box" type="text" placeholder="username"/>
        </div>
    }
}

#[component]
pub fn ChatWindow(
    messages_state: (ReadSignal<Vec<ChatMessage>>, WriteSignal<Vec<ChatMessage>>),
    joined_state: (ReadSignal<bool>, WriteSignal<bool>),
) -> impl IntoView {
    let (get_message_state, _) = messages_state;
    let messages = move || {
        get_message_state
            .get()
            .iter()
            .map(|message| (message.clone()))
            .collect::<Vec<_>>()
    };

    let (get_joined, _) = joined_state;
    let joined = move || get_joined.get();
    view! {
        <div>
        {move || if joined() {
        view! {

        <ul class="messages" style="width:600px; box-sizing: border-box; border: 1px solid black">

        <p>Messages: </p>
        <For
        each=messages
        key=move |message| {message.timestamp}
        children=move |item| {
        view! {
             <p>{item.msg}</p>
        }
        }/>
        </ul>
        }
        }else {
        view!{
        <ul></ul>
        }
        }}
        </div>
    }
}

#[component]
pub fn NewMessageInput(
    joined_state: (ReadSignal<bool>, WriteSignal<bool>),
    ws_state: (
        ReadSignal<Option<WebSocket>>,
        WriteSignal<Option<WebSocket>>,
    ),
) -> impl IntoView {
    let send_message = {
        move |event: KeyboardEvent| {
            let (ws_get, _) = ws_state;
            let ws = move || ws_get.get();
            console_log(&format!("{:?}", &ws()));
            if event.key() == "Enter" && ws().is_some() {
                let message = event_target_value(&event);
                let _ = ws().unwrap().send_with_str(&message);
            }
        }
    };
    let (get_joined, _) = joined_state;
    let joined = move || get_joined.get();
    view! {
            <div>
                {move ||
                    if joined() {
                        view! {
                            <input id="input" on:keydown=send_message style="display:block; width:600px; box-sizing: border-box" type="text" placeholder="chat"/>
                        }
                    }else {
                        view! {
                            <input id="input" on:keydown=send_message style="display:none;"/>
                        }
                    }
                }
            </div>
    }
}
