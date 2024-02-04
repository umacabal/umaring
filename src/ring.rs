use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

        let mut ring = Self {
            mapping: vec![],
            members: users,
        };

        ring.shuffle();

        ring
    }

    pub fn shuffle(&mut self) {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::new(1, 0))
            .as_secs()
            / (60 * 60 * 24 * 7); // Seed is weeks since 1970

        let mut rng = ChaChaRng::seed_from_u64(epoch);

        let mut mapping: Vec<usize> = (0..self.members.len()).collect();
        mapping.shuffle(&mut rng);

        self.mapping = mapping;
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
