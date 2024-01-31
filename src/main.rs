use chrono::Utc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    //open a listening socket
    let listeren = TcpListener::bind("localhost:8080").await.unwrap();
    let current_time = Utc::now();

    //create a listener that can receive and write messages to all connected sockets
    let (tx, _rx) = broadcast::channel(10);

    //create a loop for async to start processing
    loop {
        //accept incoming socket connections
        let (mut socket, addr) = listeren.accept().await.unwrap();

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
