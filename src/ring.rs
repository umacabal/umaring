use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Member {
    id: String,
    name: String,
    pub url: String,
}

#[derive(Clone)]
pub struct Ring {
    lookup: HashMap<String, usize>,
    members: Vec<Member>,
}

#[derive(Deserialize)]
pub struct RingSource {
    pub users: Vec<Member>,
}

impl Ring {
    pub fn new(toml: &str) -> Self {

        let mut lookup = HashMap::new();
        let mut members = Vec::new();

        let ring_source: RingSource = toml::from_str(&toml).unwrap();

        for (index, member) in ring_source.users.into_iter().enumerate() {
            lookup.insert(member.id.clone(), index);
            members.push(member);
        }

        Self { lookup, members }
    }

    pub fn get(&self, id: &str) -> Option<&Member> {
        let index = self.lookup.get(id)?;
        Some(&self.members[*index])
    }

    pub fn neighbors(&self, id: &str) -> Option<(&Member, &Member)> {
        let index = self.lookup.get(id)?;
        let prev = if *index == 0 {
            &self.members[self.members.len() - 1]
        } else {
            &self.members[*index - 1]
        };
        let next = if *index == self.members.len() - 1 {
            &self.members[0]
        } else {
            &self.members[*index + 1]
        };
        Some((prev, next))
    }

    pub fn all(&self) -> &Vec<Member> {
        &self.members
    }
}
