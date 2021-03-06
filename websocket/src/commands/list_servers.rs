use tokio::sync::RwLock;
use crate::state::State;
use tungstenite::Message;
use uuid::Uuid;
use crate::encode_cmd;
use crate::commands::ServerCommand;

pub async fn list_servers(state: &RwLock<State>, client_id: &Uuid) -> Option<Message> {
    let state = state.read().await;
    Some(Message::from(encode_cmd(
        &ServerCommand::ServerList(state.servers.values().map(|srv| {
            srv.info()
        }).collect())
    )))
}