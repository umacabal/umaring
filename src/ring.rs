use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Member {
    id: String,
    name: String,
    url: String,
}

#[derive(Clone, Debug)]
pub struct MemberLink {
    pub index: usize,
    pub member: Member,
    pub next: Option<Arc<RwLock<MemberLink>>>,
    pub prev: Option<Arc<RwLock<MemberLink>>>,
}

pub struct Ring {
    pub members: HashMap<String, Arc<RwLock<MemberLink>>>,
    pub all: Vec<Member>,
}

#[derive(Deserialize)]
pub struct RingSource {
    pub users: Vec<Member>,
}

impl Ring {
    pub fn new() -> Self {
        Ring {
            members: HashMap::new(),
            all: Vec::new(),
        }
    }

    pub async fn initialize_from_toml(&mut self, toml: &str) {
        let ring: RingSource = toml::from_str(&toml).unwrap();
        for (index, member) in ring.users.into_iter().enumerate() {
            self.add_member(member, index);
        }
    }

    pub fn add_member(&mut self, member: Member, index: usize) {
        let member_link = MemberLink {
            index,
            member: member.clone(),
            next: None,
            prev: None,
        };
        self.members.insert(member_link.member.id.clone(), Arc::new(RwLock::new(member_link)));
        self.all.push(member);
    }

    pub async fn link_members(&mut self) {
        // Collect member IDs and their indices
        let mut members_with_index: Vec<(String, usize)> = Vec::new();
        for (id, member_link) in &self.members {
            let index = member_link.read().await.index;
            members_with_index.push((id.clone(), index));
        }
    
        // Sort members by index
        members_with_index.sort_by_key(|&(_, index)| index);
    
        let len = members_with_index.len();
    
        for i in 0..len {
            let current_id = &members_with_index[i].0;
            let next_id = &members_with_index[(i + 1) % len].0;
            let prev_id = &members_with_index[(i + len - 1) % len].0;
    
            let current_member = Arc::clone(self.members.get(current_id).unwrap());
            let next_member = Arc::clone(self.members.get(next_id).unwrap());
            let prev_member = Arc::clone(self.members.get(prev_id).unwrap());
    
            {
                let mut member = current_member.write().await;
                member.next = Some(Arc::clone(&next_member));
                member.prev = Some(Arc::clone(&prev_member));
            }
        }
    }
}
