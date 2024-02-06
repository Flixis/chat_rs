use chrono::Utc;
use log::info;
use std::{collections::HashMap, io::Cursor, net::SocketAddr};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};
use uuid::Uuid;

mod logging_settings;

#[derive(Debug)]
struct Chatroom {
    id: Uuid,
    channel_name: String,
    users: HashMap<Uuid, SocketAddr>,
}

impl Chatroom {
    fn new(channel_name: String) -> Chatroom {
        Chatroom {
            id: Uuid::new_v4(),
            channel_name,
            users: HashMap::new(),
        }
    }

    fn add_user(&mut self, socketaddr: SocketAddr) {
        let user_id = Uuid::new_v4();
        self.users.insert(user_id, socketaddr);
    }
}

#[tokio::main]
async fn main() {
    logging_settings::setup_loggers();
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let (tx, _) = broadcast::channel(10);
    let mut chatroom = Chatroom::new("chatroom".to_string());

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let welcome_message = format!("{}\nWelcome to the chat server\n", Uuid::new_v4());
        let mut buffer = Cursor::new(welcome_message);
        let _ = socket.write_all_buf(&mut buffer).await;

        chatroom.add_user(addr);
        info!("{}({}) | Current users: {:?}", chatroom.id, chatroom.channel_name, chatroom.users);

        let tx = tx.clone();
        tokio::spawn(handle_connection(socket, addr, tx));
    }
}

async fn handle_connection(mut socket: tokio::net::TcpStream, addr: SocketAddr, tx: broadcast::Sender<(String, SocketAddr)>) {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break;
                }
                let timed_message = format!("{}: {}", Utc::now(), line.clone());
                tx.send((timed_message, addr)).unwrap();
                line.clear();
            }

            result = rx.recv() => {
                let (msg, other_addr) = result.unwrap();
                if addr != other_addr {
                    writer.write_all(msg.as_bytes()).await.unwrap();
                }
            }
        }
    }
}
