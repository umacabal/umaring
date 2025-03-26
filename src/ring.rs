use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Member {
    id: String,
    name: String,
    pub url: String,
}

pub struct Ring {
    members: Vec<Member>,
}

#[derive(Deserialize)]
pub struct RingSource {
    pub users: Vec<Member>,
}

impl Ring {
    pub fn new(toml: &str) -> Self {
        let ring_source: RingSource = toml::from_str(toml).unwrap();
        let users = ring_source.users;

        Self { members: users }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Member> {
        self.members.iter()
    }

    pub fn get(&self, id: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    pub fn neighbors(&self, id: &str) -> Option<(&Member, &Member)> {
        let index = self.members.iter().position(|m| m.id == id)?;
        let len = self.members.len();

        let prev = &self.members[(index + len - 1) % len];
        let next = &self.members[(index + 1) % len];

        Some((prev, next))
    }
}
