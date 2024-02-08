use chrono::Utc;
use uuid::Uuid;
use std::{net::SocketAddr};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::{broadcast},
};

use crate::{chatroom::Chatroom, commands::{Command, GreetCommand, RemoveUserByUuid}};

pub async fn handle_connection(
    mut socket: tokio::net::TcpStream,
    addr: SocketAddr,
    tx: broadcast::Sender<(String, SocketAddr)>,
    mut chatroom: Chatroom,
) {
    let (reader, mut writer) = socket.split();
    let mut reader: BufReader<tokio::net::tcp::ReadHalf<'_>> = BufReader::new(reader);
    let mut line: String = String::new();
    let mut rx: broadcast::Receiver<(String, SocketAddr)> = tx.subscribe();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break;
                }
                let trimmed_line = line.trim();
                if trimmed_line.starts_with("/hello") {
                    // Respond directly to the command without broadcasting
                    writer.write_all("world\n".as_bytes()).await.unwrap();
                    let greet_command = GreetCommand::new();
                    greet_command.execute(&mut chatroom, "".to_string());
                } else if trimmed_line.starts_with("/remove ") {
                    // Extract the UUID from the command input
                    let uuid_arg = trimmed_line.strip_prefix("/remove ").unwrap_or("").to_string();
                    if let Ok(_) = Uuid::parse_str(&uuid_arg) {
                        let remove_command = RemoveUserByUuid::new();
                        remove_command.execute(&mut chatroom, uuid_arg);
                        // Optional: send confirmation message to the admin/user who issued the command
                        writer.write_all("User removed.\n".as_bytes()).await.unwrap();
                    } else {
                        writer.write_all("Invalid UUID format.\n".as_bytes()).await.unwrap();
                    }
                } else { 
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

