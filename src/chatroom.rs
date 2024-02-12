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

    pub fn add_user(&mut self, stream: Arc<Mutex<TcpStream>>) -> Uuid {
        let user_id = Uuid::new_v4();
        // Insert user into users map
        self.users.insert(user_id, stream);
        user_id // Return the Uuid of the newly added user
    }

    pub async fn remove_user_by_uuid(&mut self, user_id: Uuid) {
        if let Some(stream) = self.to_owned().users.get(&user_id) {
            let mut stream = stream.lock().await;
            self.users.remove(&user_id);
            let _ = stream.shutdown();
        } else {
            warn!("Couldn't remove user {user_id}")
        }
    }
}
