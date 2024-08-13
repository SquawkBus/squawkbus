use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, ErrorKind, Result};

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Role: u8 {
        const Subscriber = 0b00000001;
        const Notifier = 0b00000010;
        const Publisher = 0b00000100;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Authorization {
    pub entitlements: HashSet<i32>,
    pub roles: Role,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub endpoint: String,
    pub authorization: HashMap<String, HashMap<String, Authorization>>,
}

impl Config {
    pub fn load(path: &str) -> Result<Config> {
        let file = fs::File::open(path)?;
        serde_yaml::from_reader(file).map_err(|e| io::Error::new(ErrorKind::Other, e))
    }
}
