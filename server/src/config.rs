use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, ErrorKind, Result};
use std::path::{Path, PathBuf};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tls {
    pub is_enabled: bool,
    pub hostname: String,
    pub certfile: PathBuf,
    pub keyfile: PathBuf,
    pub password: Option<String>,
}

fn default_tls() -> Tls {
    Tls {
        is_enabled: false,
        hostname: String::from("host.example.com"),
        certfile: PathBuf::from("host.crt"),
        keyfile: PathBuf::from("host.key"),
        password: None,
    }
}

fn default_endpoint() -> String {
    String::from("0.0.0.0:8080")
}

fn default_authorization() -> HashMap<String, HashMap<String, Authorization>> {
    // The default is to authorize all users for all roles on "PUB.*".
    HashMap::from([(
        String::from(".*"),
        HashMap::from([(
            String::from("PUB\\..*"),
            Authorization {
                entitlements: HashSet::from([0]),
                roles: Role::Publisher | Role::Subscriber | Role::Notifier,
            },
        )]),
    )])
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    #[serde(default = "default_tls")]
    pub tls: Tls,

    #[serde(default = "default_authorization")]
    pub authorization: HashMap<String, HashMap<String, Authorization>>,
}

impl Config {
    pub fn load<P>(path: P) -> Result<Config>
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path)?;
        serde_yaml::from_reader(file).map_err(|e| io::Error::new(ErrorKind::Other, e))
    }

    pub fn default() -> Self {
        Config {
            endpoint: default_endpoint(),
            tls: default_tls(),
            authorization: default_authorization(),
        }
    }
}
