use chrono::Utc;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::broadcast,
};

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