use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Member {
    id: String,
    name: String,
    pub url: String,
}

pub struct Ring {
    members: Vec<Member>,
    mapping: Vec<usize>,
}

#[derive(Deserialize)]
pub struct RingSource {
    pub users: Vec<Member>,
}

impl Ring {
    pub fn new(toml: &str) -> Self {
        let ring_source: RingSource = toml::from_str(toml).unwrap();
        let users = ring_source.users;

        let mapping: Vec<usize> = (0..users.len()).collect();

        Self {
            mapping,
            members: users,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Member> {
        let len = self.members.len();
        (0..len).map(move |i| &self.members[self.mapping[i % len]])
    }

    pub fn get(&self, id: &str) -> Option<&Member> {
        self.iter().find(|m| m.id == id)
    }

    pub fn neighbors(&self, id: &str) -> Option<(&Member, &Member)> {
        let index = self.iter().position(|m| m.id == id)?;

        let prev =
            &self.members[self.mapping[(index + self.members.len() - 1) % self.members.len()]];
        let next = &self.members[self.mapping[(index + 1) % self.members.len()]];

        Some((prev, next))
    }
}
