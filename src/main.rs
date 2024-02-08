use log::info;
use std::{io::Cursor, sync::Arc};
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::{broadcast, Mutex}};
use uuid::Uuid;

mod chatroom;
mod commands;
mod connection_handler;
mod logging_settings;

#[tokio::main]
async fn main() {
    logging_settings::setup_loggers();
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let (tx, _) = broadcast::channel(10);
    let chatroom = Arc::new(Mutex::new(chatroom::Chatroom::new("chatroom".to_string())));
    
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let welcome_message = format!("{}\nWelcome to the chat server\n", Uuid::new_v4());
        let mut buffer = Cursor::new(welcome_message);
        let _ = socket.write_all_buf(&mut buffer).await;
        
        let mut chatroom: tokio::sync::MutexGuard<'_, chatroom::Chatroom> = chatroom.lock().await;
        let _: Uuid = chatroom.add_user(addr);
        info!(
            "{}({}) | Current users: {:?}",
            chatroom.id, chatroom.channel_name, chatroom.users
        );
        

        let tx = tx.clone();
        let chatroom_clone = chatroom.clone();
        tokio::spawn(async move {
            // Now you can access and modify `chatroom` safely within the task.
            connection_handler::handle_connection(socket, addr, tx, chatroom_clone).await;
        });
    }
}
