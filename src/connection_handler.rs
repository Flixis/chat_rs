use chrono::Utc;
use log::info;
use std::{io::Cursor, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::{broadcast, Mutex},
};
use uuid::Uuid;

use crate::{
    chatroom::Chatroom,
    commands::{Command, GreetCommand, RemoveUserByUuid},
};

pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    tx: broadcast::Sender<(String, SocketAddr)>,
    mut chatroom: Chatroom,
) {
    let stream = Arc::new(Mutex::new(stream));
 
    {
        let stream = stream.clone();
        let _: Uuid = chatroom.add_user(stream).await;
    }
    
    let stream_clone = stream.clone();
    let mut stream = stream_clone.lock().await;
    let (reader, mut writer) = stream.split();
   

    let mut reader = BufReader::new(reader);
    let mut line: String = String::new();
    let mut rx: broadcast::Receiver<(String, SocketAddr)> = tx.subscribe();

    let welcome_message = format!("{}\nWelcome to the chat server\n", Uuid::new_v4());
    let mut buffer = Cursor::new(welcome_message);
    let _ = writer.write_all_buf(&mut buffer).await;

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break;
                }
                let trimmed_line = line.trim();
                if trimmed_line.starts_with("/") {
                    parse_command(trimmed_line, chatroom.clone(), &mut writer).await;
                }
                else {
                    // Broadcast non-command messages
                    let timed_message = format!("{}: {}", Utc::now(), line);
                    tx.send((timed_message, addr)).unwrap();
                }
                line.clear();
            }
            result = rx.recv() => {
                if let Ok((msg, other_addr)) = result {
                    // Send received messages to everyone except the sender of a command
                    if addr != other_addr {
                        writer.write_all(msg.as_bytes()).await.unwrap();
                    }
                }
            }
        }
    }
}

async fn parse_command(
    line: &str,
    mut chatroom: Chatroom,
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
) {
    if line.starts_with("/hello") {
        // Respond directly to the command without broadcasting
        writer.write_all("world\n".as_bytes()).await.unwrap();
        let greet_command = GreetCommand::new();
        greet_command.execute(&mut chatroom, "".to_string()).await;
    } else if line.starts_with("/remove ") {
        // Extract the UUID from the command input
        let uuid_arg = line.strip_prefix("/remove ").unwrap_or("").to_string();
        if let Ok(_) = Uuid::parse_str(&uuid_arg) {
            let remove_command = RemoveUserByUuid::new();
            remove_command.execute(&mut chatroom, uuid_arg).await;
            // Optional: send confirmation message to the admin/user who issued the command
            writer
                .write_all("User removed.\n".as_bytes())
                .await
                .unwrap();
        } else {
            writer
                .write_all("Invalid UUID format.\n".as_bytes())
                .await
                .unwrap();
        }
    } else if line.starts_with("/connected") {
        info!(
            "{}({}) \n Current users: {:?}",
            chatroom.id, chatroom.channel_name, chatroom.users.len()
        );
    }
}
