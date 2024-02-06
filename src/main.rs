use log::info;
use std::io::Cursor;
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::broadcast,
};
use uuid::Uuid;

mod logging_settings;
mod connection_handler;
mod chatroom;


#[tokio::main]
async fn main() {
    logging_settings::setup_loggers();
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let (tx, _) = broadcast::channel(10);
    let mut chatroom = chatroom::Chatroom::new("chatroom".to_string());

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let welcome_message = format!("{}\nWelcome to the chat server\n", Uuid::new_v4());
        let mut buffer = Cursor::new(welcome_message);
        let _ = socket.write_all_buf(&mut buffer).await;

        chatroom.add_user(addr);
        info!("{}({}) | Current users: {:?}", chatroom.id, chatroom.channel_name, chatroom.users);

        let tx = tx.clone();
        tokio::spawn(connection_handler::handle_connection(socket, addr, tx));
    }
}


