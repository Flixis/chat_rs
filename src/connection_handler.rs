use chrono::Utc;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::broadcast,
};

use crate::commands::{Command, GreetCommand};

pub async fn handle_connection(mut socket: tokio::net::TcpStream, addr: SocketAddr, tx: broadcast::Sender<(String, SocketAddr)>) {
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
                let is_command = line.trim() == "/hello";

                if is_command {
                    // Respond directly to the command without broadcasting
                    writer.write_all("world\n".as_bytes()).await.unwrap();
                    let greet_command = GreetCommand::new();
                    greet_command.execute();
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
