use log::info;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{broadcast, Mutex},
};

mod chatroom;
mod commands;
mod connection_handler;
mod logging_settings;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:8080";
    logging_settings::setup_loggers();

    let listener = TcpListener::bind(addr).await.unwrap();
    let (tx, _) = broadcast::channel(10);
    let chatroom = Arc::new(Mutex::new(chatroom::Chatroom::new("chatroom".to_string())));

    info!("Server started on {:?}", addr);
    info!("Chatrooom created with {:?}", chatroom);

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        let chatroom: tokio::sync::MutexGuard<'_, chatroom::Chatroom> = chatroom.lock().await;

        let tx = tx.clone();
        let chatroom_clone = chatroom.clone();
        tokio::spawn(async move {
            // Now you can access and modify `chatroom` safely within the task.
            connection_handler::handle_connection(stream, addr, tx, chatroom_clone).await;
        });
    }
}
