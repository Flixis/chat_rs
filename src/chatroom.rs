use std::collections::HashMap;
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Debug)]
pub struct Chatroom {
    pub id: Uuid,
    pub channel_name: String,
    pub users: HashMap<Uuid, SocketAddr>,
}

impl Chatroom {
    pub fn new(channel_name: String) -> Chatroom {
        Chatroom {
            id: Uuid::new_v4(),
            channel_name,
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, socketaddr: SocketAddr) {
        let user_id = Uuid::new_v4();
        self.users.insert(user_id, socketaddr);
    }
}
