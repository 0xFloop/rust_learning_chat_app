use chat_frontend::*;
use leptos::*;
#[component]
fn App() -> impl IntoView {
    let _mesages = create_signal::<Vec<String>>(vec![]);
    view! {
        <div class="message-app">

        <h1>"This is a basic chat app written entirely in rust"</h1>
        <UsernameInput/>
        <ChatWindow/>
        <NewMessageInput/>
        </div>
    }
}
fn main() {
    mount_to_body(App);
}
