use log::warn;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Chatroom {
    pub id: Uuid,
    pub channel_name: String,
    pub users: HashMap<Uuid, Arc<Mutex<TcpStream>>>,
}

impl Chatroom {
    pub fn new(channel_name: String) -> Self {
        Chatroom {
            id: Uuid::new_v4(),
            channel_name,
            users: HashMap::new(),
        }
    }

    pub async fn add_user(&mut self, stream: Arc<Mutex<TcpStream>>) -> Uuid {
        let user_id = Uuid::new_v4();
        println!("{user_id}");
        self.users.insert(user_id, stream);
        user_id 
    }

    pub async fn remove_user_by_uuid(&mut self, user_id: Uuid) {
        if let Some(stream) = self.users.to_owned().get(&user_id) {
            let mut stream = stream.lock().await;
            // Use tokio::io::AsyncWriteExt for the shutdown method
            if let Err(e) = stream.shutdown().await {
                warn!("Failed to shut down stream: {:?}", e);
            }
            self.users.remove(&user_id);
        } else {
            warn!("Couldn't remove user {user_id}");
        }
    }
}
