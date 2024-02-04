use chat_frontend::*;
use leptos::*;
use web_sys::WebSocket;
#[component]
fn App() -> impl IntoView {
    let messages = create_signal::<Vec<ChatMessage>>(vec![]);
    let joined = create_signal::<bool>(false);
    let ws = create_signal::<Option<WebSocket>>(None);
    view! {
        <div class="message-app">
        <h1>"This is a basic chat app written entirely in rust"</h1>
        <UsernameInput ws_state={ws} messages_state={messages} joined_state={joined}/>
        <ChatWindow messages_state={messages} joined_state={joined}/>
        <NewMessageInput ws_state={ws} joined_state={joined}/>
        </div>
    }
}
fn main() {
    mount_to_body(move || App);
}
