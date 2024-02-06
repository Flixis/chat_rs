use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Debug)]
pub struct Chatroom {
    pub id: Uuid,
    pub channel_name: String,
    pub users: HashMap<Uuid, SocketAddr>, // Maps Uuid to SocketAddr
    addr_to_users: HashMap<SocketAddr, HashSet<Uuid>>, // Maps SocketAddr to multiple Uuids
}

impl Chatroom {
    pub fn new(channel_name: String) -> Self {
        Chatroom {
            id: Uuid::new_v4(),
            channel_name,
            users: HashMap::new(),
            addr_to_users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, socketaddr: SocketAddr) -> Uuid {
        let user_id = Uuid::new_v4();
        // Insert user into users map
        self.users.insert(user_id, socketaddr);
        // Insert or update the addr_to_users map
        self.addr_to_users.entry(socketaddr).or_insert_with(HashSet::new).insert(user_id);
        user_id // Return the Uuid of the newly added user
    }

    pub fn remove_user_by_uuid(&mut self, user_id: Uuid) {
        if let Some(socketaddr) = self.users.remove(&user_id) {
            // Attempt to remove user from addr_to_users map
            if let Some(user_set) = self.addr_to_users.get_mut(&socketaddr) {
                user_set.remove(&user_id);
                // If the set is empty after removal, also remove the socketaddr entry
                if user_set.is_empty() {
                    self.addr_to_users.remove(&socketaddr);
                }
            }
        }
    }

    pub fn remove_users_by_socketaddr(&mut self, socketaddr: SocketAddr) {
        if let Some(user_ids) = self.addr_to_users.remove(&socketaddr) {
            // Remove all users associated with this SocketAddr
            for user_id in user_ids {
                self.users.remove(&user_id);
            }
        }
    }
}
