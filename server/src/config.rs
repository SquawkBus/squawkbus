use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, ErrorKind, Result};
use std::path::Path;

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

pub fn load_authorizations<P>(
    path: Option<P>,
) -> Result<HashMap<String, HashMap<String, Authorization>>>
where
    P: AsRef<Path>,
{
    match path {
        Some(path) => {
            let file = fs::File::open(path)?;
            serde_yaml::from_reader(file).map_err(|e| io::Error::new(ErrorKind::Other, e))
        }
        None => Ok(default_authorization()),
    }
}
