use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum HealthStatus {
    Unknown,
    /// Uses <script src="https://umaring.mkr.cx/ring.js">
    HealthyRingJs,
    /// Uses JavaScript to fetch https://umaring.mkr.cx/:id
    HealthyApiJs,
    /// Uses redirect links like https://umaring.mkr.cx/:id/prev or /next
    HealthyRedirectLinks,
    /// Server-side integration or static HTML containing umaring
    HealthyStatic,
    /// Found umaring reference in a linked JS file (but not ring.js or API pattern)
    HealthyJsOther,
    UnhealthyDown,
    UnhealthyMissing,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(
            self,
            HealthStatus::Unknown
                | HealthStatus::HealthyRingJs
                | HealthStatus::HealthyApiJs
                | HealthStatus::HealthyRedirectLinks
                | HealthStatus::HealthyStatic
                | HealthStatus::HealthyJsOther
        )
    }

    pub fn description(&self) -> &'static str {
        match self {
            HealthStatus::Unknown => "Not yet scanned",
            HealthStatus::HealthyRingJs => "Uses ring.js script",
            HealthStatus::HealthyApiJs => "Uses JavaScript API fetch",
            HealthStatus::HealthyRedirectLinks => "Uses prev/next redirect links",
            HealthStatus::HealthyStatic => "Server-side or static HTML integration",
            HealthStatus::HealthyJsOther => "Found in linked JavaScript",
            HealthStatus::UnhealthyDown => "Site is down or unreachable",
            HealthStatus::UnhealthyMissing => "Site is up but no umaring integration found",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MemberHealth {
    pub status: HealthStatus,
    pub last_checked: Option<u64>, // Unix timestamp
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub url: String,
}

pub struct Ring {
    members: Vec<Member>,
    health: Vec<MemberHealth>,
    mapping: Vec<usize>,
    check_index: usize,
}

#[derive(Deserialize)]
pub struct RingSource {
    pub users: Vec<Member>,
}

impl Ring {
    pub fn new(toml: &str) -> Self {
        let ring_source: RingSource = toml::from_str(toml).unwrap();
        let users = ring_source.users;
        let health = vec![
            MemberHealth {
                status: HealthStatus::Unknown,
                last_checked: None,
            };
            users.len()
        ];

        let mut ring = Self {
            mapping: vec![],
            members: users,
            health,
            check_index: 0,
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

    /// Returns an iterator over healthy members in shuffled order
    pub fn iter(&self) -> impl Iterator<Item = &Member> {
        self.healthy_indices()
            .into_iter()
            .map(move |i| &self.members[i])
    }

    /// Returns the indices of healthy members in shuffled order
    fn healthy_indices(&self) -> Vec<usize> {
        self.mapping
            .iter()
            .filter(|&&i| self.health[i].status.is_healthy())
            .copied()
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Member> {
        // Allow lookup of any member, even unhealthy ones (so they can check their own status)
        self.members.iter().find(|m| m.id == id)
    }

    pub fn neighbors(&self, id: &str) -> Option<(&Member, &Member)> {
        let healthy = self.healthy_indices();

        // First check if the member exists at all
        let member_idx = self.members.iter().position(|m| m.id == id)?;

        // Check if member is in the healthy list
        if let Some(index) = healthy.iter().position(|&i| i == member_idx) {
            // Member is healthy, return their actual neighbors
            let prev_idx = healthy[(index + healthy.len() - 1) % healthy.len()];
            let next_idx = healthy[(index + 1) % healthy.len()];
            return Some((&self.members[prev_idx], &self.members[next_idx]));
        }

        // Member is unhealthy - find where they would be in the ring
        // and return the neighbors they would have if they were healthy
        let member_mapping_pos = self.mapping.iter().position(|&i| i == member_idx)?;

        // Find the prev healthy member (search backwards in mapping)
        let mut prev_idx = None;
        for offset in 1..=self.mapping.len() {
            let check_pos = (member_mapping_pos + self.mapping.len() - offset) % self.mapping.len();
            let check_idx = self.mapping[check_pos];
            if self.health[check_idx].status.is_healthy() {
                prev_idx = Some(check_idx);
                break;
            }
        }

        // Find the next healthy member (search forwards in mapping)
        let mut next_idx = None;
        for offset in 1..=self.mapping.len() {
            let check_pos = (member_mapping_pos + offset) % self.mapping.len();
            let check_idx = self.mapping[check_pos];
            if self.health[check_idx].status.is_healthy() {
                next_idx = Some(check_idx);
                break;
            }
        }

        match (prev_idx, next_idx) {
            (Some(p), Some(n)) => Some((&self.members[p], &self.members[n])),
            _ => None, // No healthy members at all
        }
    }

    /// Get the next member to check and advance the index
    pub fn next_member_to_check(&mut self) -> &Member {
        let member = &self.members[self.check_index];
        self.check_index = (self.check_index + 1) % self.members.len();
        member
    }

    /// Update the health status for a member by their index in the original list
    pub fn set_health(&mut self, id: &str, status: HealthStatus) {
        if let Some(idx) = self.members.iter().position(|m| m.id == id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::new(0, 0))
                .as_secs();
            self.health[idx] = MemberHealth {
                status,
                last_checked: Some(now),
            };
        }
    }

    /// Get all members with their health status (for status endpoint)
    pub fn all_with_health(&self) -> Vec<(&Member, &MemberHealth)> {
        self.mapping
            .iter()
            .map(|&i| (&self.members[i], &self.health[i]))
            .collect()
    }
}
