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

    // Modify the add_user method to accept a Uuid as an identifier for the TcpStream
    fn add_user(&mut self, socketaddr: SocketAddr) {
        let user_id = Uuid::new_v4();
        self.users.insert(user_id, socketaddr);
    }
}

#[tokio::main]
async fn main() {
    logging_settings::setup_loggers();
    //open a listening socket
    let listeren = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let current_time = Utc::now();
    let uuid = Uuid::new_v4();

    //create a listener that can receive and write messages to all connected sockets
    let (tx, _rx) = broadcast::channel(10);
    let mut chatroom = Chatroom::new("chatroom".to_string());

    //create a loop for async to start processing
    loop {
        //accept incoming socket connections
        let (mut socket, addr) = listeren.accept().await.unwrap();
        let welcome_message = format!("{uuid}\nWelcome to the chat server\n");
        let mut buffer = Cursor::new(welcome_message);
        let _welcome_message = socket.write_buf(&mut buffer).await;

        chatroom.add_user(addr);
        info!("{}({}) | Current users: {:?}",chatroom.id, chatroom.channel_name, chatroom.users);

        //create a transmit and receive buffer
        let tx = tx.clone();
        //tx subscribe is for some reason the the way you read receive messages using broadcast
        let mut rx = tx.subscribe();

        //using tokio spawn we can do async things
        tokio::spawn(async move {
            //split the socket in 2 halfs, one half reads the other writes
            let (reader, mut writer) = socket.split();
            //assign the incoming data read to reader
            let mut reader = BufReader::new(reader);
            //setup empty string to put reader in buffer
            let mut line = String::new();

            //using tokio select we can do 2 things at the same time
            loop {
                tokio::select! {
                    //if we are reading from a socket
                    result = reader.read_line(&mut line) => {
                        //if we get nothing back from socket its probably disconnected so we disconnect the socket
                        if result.unwrap() == 0{
                            break;
                        }
                        //oterwise broadcast the incoming data to the whole TCP connection
                        let timed_message = format!("{current_time}: {}", line.clone());
                        tx.send((timed_message, addr)).unwrap();
                        //line clear to empty string buffer
                        line.clear();
                    }

                    //when receiving from TCP connection
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();

                        //if the data comes from the same address we should not reprint the data on that addr screen.
                        if addr != other_addr{
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
